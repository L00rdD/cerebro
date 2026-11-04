//! Menu bar (macOS) / system tray (Windows, Linux) icon.

use tauri::AppHandle;
use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem};
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
        .on_menu_event(|app, event| on_menu_event(app, &event))
        .on_tray_icon_event(|tray, event| on_icon_event(tray.app_handle(), &event))
        .build(app)?;
    Ok(())
}

fn on_menu_event(app: &AppHandle, event: &MenuEvent) {
    if event.id() == "quit" {
        app.exit(0);
    }
}

fn on_icon_event(app: &AppHandle, event: &TrayIconEvent) {
    tauri_plugin_positioner::on_tray_event(app, event);
    if is_left_click(event) {
        panel::toggle(app);
    }
}

/// A left click, once the button is released, toggles the panel.
fn is_left_click(event: &TrayIconEvent) -> bool {
    matches!(
        event,
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
    )
}

/// Where the icon sits on screen, in physical pixels (unknown on Linux).
pub fn rect(app: &AppHandle) -> Option<tauri::Rect> {
    app.tray_by_id(TRAY_ID)?.rect().ok().flatten()
}

/// Filled icon while something is working; character count next to it (macOS only).
pub fn update(app: &AppHandle, counts: &CountsDto) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    if let Ok(icon) = Image::from_bytes(icon_bytes(counts)) {
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_icon_as_template(true);
    }
    let _ = tray.set_title(title(counts).as_deref());
    let _ = tray.set_tooltip(Some(tooltip(counts)));
}

fn icon_bytes(counts: &CountsDto) -> &'static [u8] {
    if counts.busy > 0 || counts.agents > 0 {
        ICON_ACTIVE
    } else {
        ICON_IDLE
    }
}

fn title(counts: &CountsDto) -> Option<String> {
    (counts.sessions > 0).then(|| counts.characters.to_string())
}

fn tooltip(counts: &CountsDto) -> String {
    format!(
        "Cerebro: {} session(s), {} busy, {} agent(s)",
        counts.sessions, counts.busy, counts.agents
    )
}

#[cfg(test)]
mod tests {
    use tauri::PhysicalPosition;
    use tauri::tray::TrayIconId;

    use super::*;

    fn counts(sessions: usize, busy: usize, agents: usize, characters: usize) -> CountsDto {
        CountsDto {
            sessions,
            busy,
            agents,
            characters,
        }
    }

    fn click(button: MouseButton, button_state: MouseButtonState) -> TrayIconEvent {
        TrayIconEvent::Click {
            id: TrayIconId::new(TRAY_ID),
            position: PhysicalPosition::new(0.0, 0.0),
            rect: tauri::Rect::default(),
            button,
            button_state,
        }
    }

    #[test]
    fn icon_is_filled_while_something_works() {
        assert_eq!(icon_bytes(&counts(2, 0, 0, 2)), ICON_IDLE);
        assert_eq!(icon_bytes(&counts(2, 1, 0, 2)), ICON_ACTIVE);
        assert_eq!(icon_bytes(&counts(2, 0, 1, 3)), ICON_ACTIVE);
    }

    #[test]
    fn title_counts_characters_once_a_session_is_live() {
        assert_eq!(title(&counts(0, 0, 0, 0)), None);
        assert_eq!(title(&counts(2, 1, 3, 5)).as_deref(), Some("5"));
    }

    #[test]
    fn tooltip_sums_up_the_counts() {
        assert_eq!(
            tooltip(&counts(2, 1, 3, 5)),
            "Cerebro: 2 session(s), 1 busy, 3 agent(s)"
        );
    }

    #[test]
    fn only_a_released_left_click_toggles_the_panel() {
        assert!(is_left_click(&click(
            MouseButton::Left,
            MouseButtonState::Up
        )));
        assert!(!is_left_click(&click(
            MouseButton::Left,
            MouseButtonState::Down
        )));
        assert!(!is_left_click(&click(
            MouseButton::Right,
            MouseButtonState::Up
        )));
        assert!(!is_left_click(&TrayIconEvent::Enter {
            id: TrayIconId::new(TRAY_ID),
            position: PhysicalPosition::new(0.0, 0.0),
            rect: tauri::Rect::default(),
        }));
    }
}
