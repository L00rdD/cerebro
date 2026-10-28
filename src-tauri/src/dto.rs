//! View models sent to the desktop panel. The domain stays free of serde; mapping happens here.
//! Snapshots use the DTOs shared with the phone page (`cerebro_web::dto`).

use std::net::SocketAddr;

use cerebro_domain::ports::{FocusOutcome, HookSetup, NetworkStatus};
use cerebro_domain::usecases::NetworkCheck;
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

/// How a phone reaches Cerebro: one entry per network, in the panel's order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileDto {
    pub networks: Vec<NetworkDto>,
}

impl MobileDto {
    /// The first link of a network, the one shared with the phone.
    pub fn link(&self, network: &str) -> Option<&str> {
        self.networks
            .iter()
            .find(|candidate| candidate.id == network)?
            .urls
            .first()
            .map(String::as_str)
    }
}

/// One network: whether it is on, its state and the links to open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDto {
    /// e.g. `zerotier`, `local`.
    pub id: &'static str,
    /// e.g. `ZeroTier`.
    pub name: String,
    /// `off`, `notInstalled`, `stopped`, `offline`, `online` or `error`.
    pub state: &'static str,
    /// One link per address, key included. Empty until the server listens.
    pub urls: Vec<String>,
    /// What went wrong (network query, port already taken…).
    pub message: Option<String>,
}

impl NetworkDto {
    pub fn new(
        check: &NetworkCheck,
        served_on: &[SocketAddr],
        key: &AccessKey,
        server_error: Option<String>,
    ) -> Self {
        let (state, message) = match &check.status {
            None => ("off", None),
            Some(Ok(NetworkStatus::NotInstalled)) => ("notInstalled", None),
            Some(Ok(NetworkStatus::Stopped)) => ("stopped", None),
            Some(Ok(NetworkStatus::Offline)) => ("offline", None),
            Some(Ok(NetworkStatus::Online(_))) => ("online", server_error),
            Some(Err(error)) => ("error", Some(error.to_string())),
        };
        Self {
            id: check.id,
            name: check.name.clone(),
            state,
            urls: served_on
                .iter()
                .map(|address| format!("http://{address}/?key={}", key.as_str()))
                .collect(),
            message,
        }
    }
}
