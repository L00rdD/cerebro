//! The popover window anchored under the tray icon.

use std::sync::PoisonError;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, LogicalSize, Manager, WebviewWindow, WindowEvent};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::state::AppState;

#[cfg(target_os = "macos")]
tauri_nspanel::tauri_panel! {
    panel!(TrayPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true
        }
    })
}

pub const LABEL: &str = "panel";
pub const WIDTH: f64 = 446.0;
/// A tray click first blurs (hides) the panel, then toggles it: ignore that toggle.
const REOPEN_DEBOUNCE: Duration = Duration::from_millis(300);

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

pub fn attach(app: &AppHandle) {
    let Some(panel) = window(app) else { return };
    #[cfg(target_os = "macos")]
    float_over_full_screen_apps(&panel);
    let handle = app.clone();
    panel.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            hide(&handle);
        }
    });
}

/// macOS never shows an ordinary window over another app's full-screen space: the panel
/// would open on the desktop, out of sight. Like the menu bar's own popovers, it becomes a
/// non-activating panel that joins every space, full-screen ones included.
#[cfg(target_os = "macos")]
fn float_over_full_screen_apps(panel: &WebviewWindow) {
    use tauri_nspanel::{CollectionBehavior, PanelLevel, StyleMask, WebviewWindowExt};

    let Ok(panel) = panel.to_panel::<TrayPanel>() else {
        return;
    };
    panel.set_level(PanelLevel::Status.value());
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .can_join_all_spaces()
            .full_screen_auxiliary()
            .value(),
    );
    let _ = panel.add_style_mask(StyleMask::empty().nonactivating_panel().value());
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
        .is_some_and(|hidden_at| hidden_at.elapsed() < REOPEN_DEBOUNCE);
    if !just_hidden {
        show(app);
    }
}

pub fn show(app: &AppHandle) {
    let Some(panel) = window(app) else { return };
    let _ = panel.move_window(Position::TrayBottomCenter);
    reveal(&panel);
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

/// Activating the app would pull the user out of a full-screen space: on macOS the panel
/// only takes the keyboard focus.
#[cfg(target_os = "macos")]
fn reveal(panel: &WebviewWindow) {
    use tauri_nspanel::ManagerExt;

    if let Ok(ns_panel) = panel.app_handle().get_webview_panel(LABEL) {
        ns_panel.show_and_make_key();
    } else {
        let _ = panel.show();
        let _ = panel.set_focus();
    }
}

#[cfg(not(target_os = "macos"))]
fn reveal(panel: &WebviewWindow) {
    let _ = panel.show();
    let _ = panel.set_focus();
}

fn set_visible(app: &AppHandle, visible: bool) {
    app.state::<AppState>()
        .panel_visible
        .store(visible, Ordering::Relaxed);
    let _ = app.emit("panel-visibility", visible);
}
