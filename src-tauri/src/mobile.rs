//! Serves Cerebro to phones on the private network (`ZeroTier`), and keeps the server in step
//! with it: started once the network is up, moved when its address changes, stopped when it
//! goes away. Phones take over a session through its tool (e.g. Claude Code's Remote Control).

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use cerebro_domain::ports::NetworkStatus;
use cerebro_web::{AccessKey, Asset, Assets, DEFAULT_PORT, WebServer};
use tauri::{AppHandle, Emitter, Manager};

use crate::dto::MobileDto;
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
    let network = state.container.network.name();
    let key = match AccessKey::load_or_create(&state.container.mobile_key_file) {
        Ok(key) => key,
        Err(error) => {
            publish(
                &app,
                &MobileDto {
                    network,
                    state: "error",
                    urls: Vec::new(),
                    message: Some(format!("cannot create the phone link key: {error}")),
                },
            );
            return;
        }
    };
    let assets: Arc<dyn Assets> = Arc::new(EmbeddedAssets(app.clone()));
    let mut server: Option<WebServer> = None;

    loop {
        let status = network_status(&app).await;
        let wanted = addresses(&status);
        let current = server
            .as_ref()
            .map(|running| running.addresses().to_vec())
            .unwrap_or_default();
        let mut server_error = None;
        if wanted != current {
            // Dropping the old server closes its listeners and event streams.
            server = None;
            if !wanted.is_empty() {
                match WebServer::start(&wanted, &state.publisher, assets.clone(), key.clone()).await
                {
                    Ok(started) => server = Some(started),
                    Err(error) => {
                        server_error = Some(format!("cannot serve the phone page: {error}"));
                    }
                }
            }
        }
        let served_on = server.as_ref().map_or(&[][..], WebServer::addresses);
        publish(
            &app,
            &MobileDto::new(network.clone(), &status, served_on, &key, server_error),
        );

        tokio::select! {
            () = tokio::time::sleep(CHECK_INTERVAL) => {}
            () = state.mobile_recheck.notified() => {}
        }
    }
}

/// `zerotier-cli` is a blocking process call: keep it off the async workers.
async fn network_status(app: &AppHandle) -> Result<NetworkStatus, String> {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<AppState>()
            .container
            .network
            .status()
            .map_err(|error| error.to_string())
    })
    .await
    .unwrap_or_else(|error| Err(error.to_string()))
}

/// Private addresses only: the network adapter never reports public ones.
fn addresses(status: &Result<NetworkStatus, String>) -> Vec<SocketAddr> {
    match status {
        Ok(NetworkStatus::Online(ips)) => ips
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
