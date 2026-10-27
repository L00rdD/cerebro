//! View models sent to the desktop panel. The domain stays free of serde; mapping happens here.
//! Snapshots use the DTOs shared with the phone page (`cerebro_web::dto`).

use std::net::SocketAddr;

use cerebro_domain::ports::{FocusOutcome, HookSetup, NetworkStatus};
use cerebro_web::AccessKey;
use serde::Serialize;

pub use cerebro_web::dto::{CountsDto, SnapshotDto};

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum FocusDto {
    Tab { app: String },
    App { app: String },
    NotFound,
}

impl From<FocusOutcome> for FocusDto {
    fn from(outcome: FocusOutcome) -> Self {
        match outcome {
            FocusOutcome::Tab { app } => Self::Tab { app },
            FocusOutcome::App { app } => Self::App { app },
            FocusOutcome::NotFound => Self::NotFound,
        }
    }
}

pub fn hook_setup_name(setup: HookSetup) -> &'static str {
    match setup {
        HookSetup::Installed => "installed",
        HookSetup::Missing => "missing",
        HookSetup::Outdated => "outdated",
    }
}

/// How a phone reaches Cerebro: the private network's state and the links to open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileDto {
    /// e.g. `ZeroTier`.
    pub network: String,
    /// `notInstalled`, `stopped`, `offline`, `online` or `error`.
    pub state: &'static str,
    /// One link per private address, key included. Empty until the server listens.
    pub urls: Vec<String>,
    /// What went wrong (network query, port already taken…).
    pub message: Option<String>,
}

impl MobileDto {
    pub fn new(
        network: String,
        status: &Result<NetworkStatus, String>,
        served_on: &[SocketAddr],
        key: &AccessKey,
        server_error: Option<String>,
    ) -> Self {
        let (state, message) = match status {
            Ok(NetworkStatus::NotInstalled) => ("notInstalled", None),
            Ok(NetworkStatus::Stopped) => ("stopped", None),
            Ok(NetworkStatus::Offline) => ("offline", None),
            Ok(NetworkStatus::Online(_)) => ("online", server_error),
            Err(error) => ("error", Some(error.clone())),
        };
        Self {
            network,
            state,
            urls: served_on
                .iter()
                .map(|address| format!("http://{address}/?key={}", key.as_str()))
                .collect(),
            message,
        }
    }
}
