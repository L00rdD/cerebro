//! Cerebro desktop shell (presentation layer).
//!
//! ```text
//! injector  builds the dependency graph (the template's GetIt equivalent)
//! state     shared app state: use cases + last snapshot
//! poller    background loop running ObserveSessions and pushing updates
//! tray      menu bar / system tray icon
//! panel     the popover window: toggle, position, hide on blur
//! commands  functions the UI can call
//! dto       serializable view models sent to the UI
//! ```

mod commands;
mod dto;
mod injector;
mod panel;
mod poller;
mod state;
mod tray;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

use crate::state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::focus_session,
            commands::open_folder,
            commands::hook_status,
            commands::install_hooks,
            commands::autostart_enabled,
            commands::set_autostart,
            commands::resize_panel,
            commands::quit,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let container = injector::build()?;
            app.manage(AppState::new(container));
            tray::create(app.handle())?;
            panel::attach(app.handle());
            poller::start(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Cerebro");
}
