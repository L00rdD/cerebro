//! Background refresh loop. EXERCISE 8 replaces most of the polling with a file watcher.

use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::dto::SnapshotDto;
use crate::state::AppState;
use crate::tray;

const VISIBLE_INTERVAL: Duration = Duration::from_secs(1);
/// Only the tray counter is visible: a slower pace is enough.
const HIDDEN_INTERVAL: Duration = Duration::from_secs(2);

pub fn start(app: AppHandle) {
    thread::Builder::new()
        .name("cerebro-poller".into())
        .spawn(move || {
            loop {
                refresh(&app);
                let visible = app
                    .state::<AppState>()
                    .panel_visible
                    .load(Ordering::Relaxed);
                thread::sleep(if visible {
                    VISIBLE_INTERVAL
                } else {
                    HIDDEN_INTERVAL
                });
            }
        })
        .expect("failed to start the poller thread");
}

/// Observes sessions once and pushes the result to the UI and the tray if it changed.
pub fn refresh(app: &AppHandle) {
    let state = app.state::<AppState>();
    match state.container.observe.execute() {
        Ok(snapshot) => {
            let dto = SnapshotDto::from(&snapshot);
            if state.replace_snapshot(snapshot) {
                tray::update(app, &dto.counts);
                let _ = app.emit("snapshot", &dto);
            }
        }
        Err(error) => eprintln!("cerebro: observe failed: {error}"),
    }
}
