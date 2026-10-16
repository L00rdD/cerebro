//! Commands callable from the UI with `invoke(...)`. Errors cross the bridge as strings.

// Tauri commands must take `AppHandle` and arguments by value.
#![allow(clippy::needless_pass_by_value)]

use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_opener::OpenerExt as _;

use crate::dto::{FocusDto, SnapshotDto, hook_setup_name};
use crate::state::AppState;
use crate::{panel, poller};

#[tauri::command]
pub fn get_snapshot(app: AppHandle, state: State<'_, AppState>) -> Option<SnapshotDto> {
    if state.snapshot().is_none() {
        poller::refresh(&app);
    }
    state.snapshot().as_ref().map(SnapshotDto::from)
}

/// Click on a card: bring its terminal tab to the front.
#[tauri::command]
pub async fn focus_session(app: AppHandle, session_id: String) -> Result<FocusDto, String> {
    // AppleScript can take a while (or wait for a permission prompt): keep it off the UI thread.
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let scene = state.scene(&session_id).ok_or_else(|| format!("unknown session {session_id}"))?;
        let outcome = state.container.focus.execute(&scene.session).map_err(|e| e.to_string())?;
        let dto = FocusDto::from(outcome);
        if !matches!(dto, FocusDto::NotFound) {
            panel::hide(&app);
        }
        Ok(dto)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn open_folder(app: AppHandle, state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    let scene = state.scene(&session_id).ok_or_else(|| format!("unknown session {session_id}"))?;
    app.opener()
        .open_path(scene.session.cwd.display().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hook_status(state: State<'_, AppState>) -> Result<&'static str, String> {
    state.container.hooks.status().map(hook_setup_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn install_hooks(state: State<'_, AppState>) -> Result<&'static str, String> {
    state.container.hooks.install().map(hook_setup_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn autostart_enabled(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let launcher = app.autolaunch();
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    result.map_err(|e| e.to_string())?;
    launcher.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resize_panel(app: AppHandle, height: f64) {
    panel::resize(&app, height);
}

#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}
