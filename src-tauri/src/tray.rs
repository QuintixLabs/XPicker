// Tray icon — click opens XPicker (click again to close it); right-click shows the menu:
//
//   Open XPicker
//   Settings
//   ─────────
//   Pick color            Ctrl+Shift+X
//   Recent colors       ▸ (last 5, each with its color; click to copy)
//   ─────────
//   Exit
//
// The menu is rebuilt whenever settings or recent colors change (see Store::update).

use tauri::{
    image::Image,
    menu::{IconMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::{clipboard::Clipboard, color, notify, picker, store::Store, ui};

const ID: &str = "xpicker";
const RECENT_COUNT: usize = 5;

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let mut tray = TrayIconBuilder::with_id(ID)
        .tooltip(tooltip(app))
        .menu(&menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                ui::toggle_main_async(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Rebuilds the menu and tooltip from the current settings and recent colors.
pub fn refresh(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id(ID) {
        if let Ok(menu) = menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
        let _ = tray.set_tooltip(Some(tooltip(app)));
    }
}

/// Hover text while something is happening ("Updating to 1.0.4…"); None goes back to normal.
pub fn set_status(app: &AppHandle, status: Option<&str>) {
    if let Some(tray) = app.tray_by_id(ID) {
        let text = status.map(|s| format!("XPicker · {s}")).unwrap_or_else(|| tooltip(app));
        let _ = tray.set_tooltip(Some(text));
    }
}

fn tooltip(app: &AppHandle) -> String {
    let shortcut = app.state::<Store>().get().settings.shortcut;
    if shortcut.is_empty() {
        "XPicker".into()
    } else {
        format!("XPicker · {} to pick", shortcut_label(&shortcut))
    }
}

/// "Ctrl+Shift+KeyX" → "Ctrl+Shift+X"
fn shortcut_label(shortcut: &str) -> String {
    shortcut
        .split('+')
        .map(|part| part.strip_prefix("Key").or_else(|| part.strip_prefix("Digit")).unwrap_or(part))
        .map(|part| if part == "Super" { if cfg!(target_os = "macos") { "Cmd" } else { "Win" } } else { part })
        .collect::<Vec<_>>()
        .join("+")
}

fn menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let data = app.state::<Store>().get();
    let shortcut = data.settings.shortcut;

    // The shortcut shows on the right of the item. If the system can't display
    // that key combination, the item is shown without it.
    let pick = MenuItem::with_id(app, "pick", "Pick color", true, Some(shortcut.as_str()))
        .or_else(|_| MenuItem::with_id(app, "pick", "Pick color", true, None::<&str>))?;

    let recent = Submenu::with_id(app, "recent", "Recent colors", !data.history.is_empty())?;
    for entry in data.history.iter().take(RECENT_COUNT) {
        let label = color::format(&entry.hex, data.settings.format);
        let item = IconMenuItem::with_id(app, format!("recent:{}", entry.hex), label, true, Some(swatch(&entry.hex)), None::<&str>)?;
        recent.append(&item)?;
    }

    let open = MenuItem::with_id(app, "open", "Open XPicker", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;

    Menu::with_items(
        app,
        &[&open, &settings, &PredefinedMenuItem::separator(app)?, &pick, &recent, &PredefinedMenuItem::separator(app)?, &exit],
    )
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "pick" => picker::start(app),
        "open" => ui::open_main_async(app),
        "settings" => ui::open_page_async(app, "settings"),
        "exit" => app.exit(0),
        _ => {
            if let Some(hex) = id.strip_prefix("recent:") {
                copy_recent(app, hex);
            }
        }
    }
}

/// Copies a recent color in the current format, like clicking it in the window.
fn copy_recent(app: &AppHandle, hex: &str) {
    let settings = app.state::<Store>().get().settings;
    let value = color::format(hex, settings.format);
    if app.state::<Clipboard>().copy(&value).is_ok() && settings.notify {
        let (app, hex) = (app.clone(), hex.to_owned());
        std::thread::spawn(move || notify::copied(&app, &value, &hex));
    }
}

/// A 16×16 square of the color for the menu.
fn swatch(hex: &str) -> Image<'static> {
    let [r, g, b] = color::rgb(hex);
    let pixels = [r, g, b, 255].repeat(16 * 16);
    Image::new_owned(pixels, 16, 16)
}
