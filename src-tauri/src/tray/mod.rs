// ============================================================
// tray/mod.rs
// The tray icon next to the clock and its menu.
// ============================================================

use tauri::{
    image::Image,
    menu::{IconMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub mod notifications;
pub mod startup;
pub mod updates;

use crate::{
    picker::{self, colors::{self, clipboard::Clipboard}},
    settings::Store,
    window::{self, logo},
};

const ID: &str = "xpicker";
const RECENT_COUNT: usize = 5;

/// Linux trays name the icon after the app's name, which is the program's
/// file name ("xpicker") unless it's set.
#[cfg(target_os = "linux")]
fn set_app_name() {
    #[link(name = "glib-2.0")]
    extern "C" {
        fn g_set_application_name(name: *const std::ffi::c_char);
    }
    unsafe { g_set_application_name(c"XPicker".as_ptr()) };
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    #[cfg(target_os = "linux")]
    set_app_name();
    let light = logo::taskbar_is_light();
    let mut tray = TrayIconBuilder::with_id(ID)
        .tooltip(tooltip(app))
        .menu(&menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                window::toggle_async(tray.app_handle());
            }
        });

    if let Some(icon) = logo::tray_icon(light).or_else(|| app.default_window_icon().cloned()) {
        tray = tray.icon(icon);
    }
    let tray = tray.build(app)?;
    let _ = tray.set_visible(shown(app));
    logo::remember_light(light);
    watch_taskbar(app, light);
    Ok(())
}

/// Swaps the tray logo when the taskbar switches between light and dark. On
/// Linux that's the desktop's theme, and the windows don't always hear about it
/// themselves, so they get told too ("system-theme").
fn watch_taskbar(app: &AppHandle, mut light: bool) {
    let app = app.clone();
    let every = std::time::Duration::from_secs(if cfg!(target_os = "linux") { 2 } else { 5 });
    std::thread::spawn(move || loop {
        std::thread::sleep(every);
        let now = logo::taskbar_is_light();
        if now != light {
            light = now;
            logo::remember_light(light);
            if let (Some(tray), Some(icon)) = (app.tray_by_id(ID), logo::tray_icon(light)) {
                let _ = tray.set_icon(Some(icon));
            }
            if cfg!(target_os = "linux") {
                let _ = app.emit("system-theme", light);
            }
        }
    });
}

/// Rebuilds the menu and tooltip from the current settings and recent colors.
pub fn refresh(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id(ID) {
        if let Ok(menu) = menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
        let _ = tray.set_tooltip(Some(tooltip(app)));
        let _ = tray.set_visible(shown(app));
    }
}

/// "Tray icon" in Settings. Hidden, XPicker is opened by starting it again
/// (it's already running, so that just opens the window).
fn shown(app: &AppHandle) -> bool {
    app.state::<Store>().get().settings.show_in_tray
}

/// Hover text while something is happening ("Updating to 1.0.4…"); None goes back to normal.
pub fn set_status(app: &AppHandle, status: Option<&str>) {
    if let Some(tray) = app.tray_by_id(ID) {
        let text = status.map(|s| format!("XPicker · {s}")).unwrap_or_else(|| tooltip(app));
        let _ = tray.set_tooltip(Some(text));
    }
}

fn tooltip(_app: &AppHandle) -> String {
    "XPicker".into()
}

fn menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let data = app.state::<Store>().get();
    let shortcut = data.settings.shortcut;

    // The shortcut shows on the right of the item. Windows menus can't show the
    // Windows key (the app would crash), so then the item is shown without it.
    let win_key = shortcut.split('+').any(|key| matches!(key, "Super" | "Meta" | "Command" | "Cmd"));
    let shown = if cfg!(windows) && win_key { None } else { Some(shortcut.as_str()) };
    let pick = MenuItem::with_id(app, "pick", "Pick color", true, shown)
        .or_else(|_| MenuItem::with_id(app, "pick", "Pick color", true, None::<&str>))?;

    let recent = Submenu::with_id(app, "recent", "Recent colors", !data.history.is_empty())?;
    for entry in data.history.iter().take(RECENT_COUNT) {
        let label = colors::format(&entry.hex, data.settings.format);
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
        "open" => window::open_async(app),
        "settings" => window::open_page_async(app, "settings"),
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
    let value = colors::format(hex, settings.format);
    if app.state::<Clipboard>().copy(&value).is_ok() && settings.notify {
        let (app, hex) = (app.clone(), hex.to_owned());
        std::thread::spawn(move || notifications::copied(&app, &value, &hex));
    }
}

/// A 16×16 square of the color for the menu.
fn swatch(hex: &str) -> Image<'static> {
    let [r, g, b] = colors::rgb(hex);
    let pixels = [r, g, b, 255].repeat(16 * 16);
    Image::new_owned(pixels, 16, 16)
}
