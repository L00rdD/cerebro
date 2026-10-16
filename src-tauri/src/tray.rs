//! Menu bar (macOS) / system tray (Windows, Linux) icon.

use tauri::AppHandle;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

use crate::dto::CountsDto;
use crate::panel;

const TRAY_ID: &str = "cerebro";
const ICON_ACTIVE: &[u8] = include_bytes!("../../assets/tray/tray-active.png");
const ICON_IDLE: &[u8] = include_bytes!("../../assets/tray/tray-idle.png");

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit Cerebro", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(ICON_IDLE)?)
        .icon_as_template(true)
        .tooltip("Cerebro")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                panel::toggle(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// Filled icon while something is working; character count next to it (macOS only).
pub fn update(app: &AppHandle, counts: &CountsDto) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let active = counts.busy > 0 || counts.agents > 0;
    if let Ok(icon) = Image::from_bytes(if active { ICON_ACTIVE } else { ICON_IDLE }) {
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_icon_as_template(true);
    }
    let title = (counts.sessions > 0).then(|| counts.characters.to_string());
    let _ = tray.set_title(title.as_deref());
    let _ = tray.set_tooltip(Some(format!(
        "Cerebro: {} session(s), {} busy, {} agent(s)",
        counts.sessions, counts.busy, counts.agents
    )));
}
