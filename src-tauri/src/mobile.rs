//! Serves Cerebro to phones on the networks the user turned on (`ZeroTier`, local network), and
//! keeps one server per network in step with it: started once the network is up, moved when
//! its address changes, stopped when it goes away or is turned off. Phones take over a session
//! through its tool (e.g. Claude Code's Remote Control).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use cerebro_domain::ports::NetworkStatus;
use cerebro_domain::usecases::NetworkCheck;
use cerebro_web::{AccessKey, Asset, Assets, DEFAULT_PORT, WebServer};
use tauri::{AppHandle, Emitter, Manager};

use crate::dto::{MobileDto, NetworkDto};
use crate::state::AppState;

/// Joining a network or getting an address takes seconds: no need to check more often.
const CHECK_INTERVAL: Duration = Duration::from_secs(10);

/// The UI files Tauri embeds in the app (read from `dist/` in dev: run `npm run build:ui`).
struct EmbeddedAssets(AppHandle);

impl Assets for EmbeddedAssets {
    fn get(&self, path: &str) -> Option<Asset> {
        let asset = self.0.asset_resolver().get(path.to_owned())?;
        Some(Asset {
            bytes: asset.bytes,
            mime_type: asset.mime_type,
        })
    }
}

pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(supervise(app));
}

async fn supervise(app: AppHandle) {
    let state = app.state::<AppState>();
    let key = match AccessKey::load_or_create(&state.container.mobile_key_file) {
        Ok(key) => key,
        Err(error) => {
            let message = format!("cannot create the phone link key: {error}");
            let networks = check_networks(&app)
                .await
                .into_iter()
                .map(|check| NetworkDto {
                    id: check.id,
                    name: check.name,
                    state: "error",
                    urls: Vec::new(),
                    message: Some(message.clone()),
                })
                .collect();
            publish(&app, &MobileDto { networks });
            return;
        }
    };
    let assets: Arc<dyn Assets> = Arc::new(EmbeddedAssets(app.clone()));
    // One server per network: a Wi-Fi change doesn't cut the phones following over ZeroTier.
    let mut servers: HashMap<&'static str, WebServer> = HashMap::new();

    loop {
        let mut networks = Vec::new();
        for check in check_networks(&app).await {
            let wanted = addresses(&check);
            let current = servers
                .get(check.id)
                .map(|running| running.addresses().to_vec())
                .unwrap_or_default();
            let mut server_error = None;
            if wanted != current {
                // Dropping the old server closes its listeners and event streams.
                servers.remove(check.id);
                if !wanted.is_empty() {
                    match WebServer::start(&wanted, &state.publisher, assets.clone(), key.clone())
                        .await
                    {
                        Ok(started) => {
                            servers.insert(check.id, started);
                        }
                        Err(error) => {
                            server_error = Some(format!("cannot serve the phone page: {error}"));
                        }
                    }
                }
            }
            let served_on = servers.get(check.id).map_or(&[][..], WebServer::addresses);
            networks.push(NetworkDto::new(&check, served_on, &key, server_error));
        }
        publish(&app, &MobileDto { networks });

        tokio::select! {
            () = tokio::time::sleep(CHECK_INTERVAL) => {}
            () = state.mobile_recheck.notified() => {}
        }
    }
}

/// `zerotier-cli` is a blocking process call: keep it off the async workers.
async fn check_networks(app: &AppHandle) -> Vec<NetworkCheck> {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || app.state::<AppState>().container.networks.check())
        .await
        .unwrap_or_default()
}

/// Private addresses only: the network adapters never report public ones.
fn addresses(check: &NetworkCheck) -> Vec<SocketAddr> {
    match &check.status {
        Some(Ok(NetworkStatus::Online(ips))) => ips
            .iter()
            .map(|ip| SocketAddr::new(*ip, DEFAULT_PORT))
            .collect(),
        _ => Vec::new(),
    }
}

fn publish(app: &AppHandle, mobile: &MobileDto) {
    if app.state::<AppState>().replace_mobile(mobile.clone()) {
        let _ = app.emit("mobile", mobile);
    }
}
