// Tray icon — click opens the main window; the menu picks or quits.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

use crate::{picker, ui};

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let pick = MenuItem::with_id(app, "pick", "Pick color", true, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Open XPicker", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&pick, &open, &separator, &quit])?;

    let mut tray = TrayIconBuilder::with_id("xpicker")
        .tooltip("XPicker")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "pick" => picker::start(app),
            "open" => ui::open_main_async(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                ui::open_main_async(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}
