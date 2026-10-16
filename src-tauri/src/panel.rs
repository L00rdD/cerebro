//! The popover window anchored under the tray icon.

use std::sync::PoisonError;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, LogicalSize, Manager, WebviewWindow, WindowEvent};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::state::AppState;

pub const LABEL: &str = "panel";
pub const WIDTH: f64 = 446.0;
/// A tray click first blurs (hides) the panel, then toggles it: ignore that toggle.
const REOPEN_DEBOUNCE: Duration = Duration::from_millis(300);

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

pub fn attach(app: &AppHandle) {
    let Some(panel) = window(app) else { return };
    let handle = app.clone();
    panel.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            hide(&handle);
        }
    });
}

pub fn toggle(app: &AppHandle) {
    let Some(panel) = window(app) else { return };
    if panel.is_visible().unwrap_or(false) {
        hide(app);
        return;
    }
    let state = app.state::<AppState>();
    let just_hidden = state
        .last_hidden
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_some_and(|at| at.elapsed() < REOPEN_DEBOUNCE);
    if !just_hidden {
        show(app);
    }
}

pub fn show(app: &AppHandle) {
    let Some(panel) = window(app) else { return };
    let _ = panel.move_window(Position::TrayBottomCenter);
    let _ = panel.show();
    let _ = panel.set_focus();
    set_visible(app, true);
    // Fresh data right away instead of waiting for the next slow tick.
    crate::poller::refresh(app);
}

pub fn hide(app: &AppHandle) {
    let Some(panel) = window(app) else { return };
    if !panel.is_visible().unwrap_or(false) {
        return;
    }
    let _ = panel.hide();
    *app.state::<AppState>()
        .last_hidden
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
    set_visible(app, false);
}

/// The UI reports its content height; the window follows and stays anchored to the tray.
pub fn resize(app: &AppHandle, height: f64) {
    let Some(panel) = window(app) else { return };
    let _ = panel.set_size(LogicalSize::new(WIDTH, height.clamp(120.0, 900.0)));
    if panel.is_visible().unwrap_or(false) {
        let _ = panel.move_window(Position::TrayBottomCenter);
    }
}

fn set_visible(app: &AppHandle, visible: bool) {
    app.state::<AppState>()
        .panel_visible
        .store(visible, Ordering::Relaxed);
    let _ = app.emit("panel-visibility", visible);
}
