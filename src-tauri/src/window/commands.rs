// ============================================================
// window/commands.rs
// Everything the windows can ask the app to do: save settings,
// copy a color, check for updates, and so on.
// ============================================================

// Anything that touches windows or the global shortcut is async: plain
// commands run on the UI thread, and doing that work there can freeze the
// app on Windows.

use tauri::{ipc::Response, AppHandle, State, WebviewWindow};

use crate::{
    picker::{self, colors::clipboard::Clipboard, Frame},
    settings::{self, Data, Settings, Store},
    tray::{startup, updates},
};

// ------------------------------------------------------------
// Settings and colors
// ------------------------------------------------------------

#[tauri::command]
pub fn state(store: State<Store>) -> Data {
    store.get()
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, store: State<'_, Store>, settings: Settings) -> Result<Data, String> {
    let previous = store.get().settings;
    if settings.shortcut != previous.shortcut {
        if let Err(err) = picker::set_shortcut(&app, &settings.shortcut) {
            let _ = picker::set_shortcut(&app, &previous.shortcut);
            return Err(format!("That shortcut can't be used ({err})."));
        }
    }
    if settings.start_with_computer != previous.start_with_computer {
        startup::set(&app, settings.start_with_computer)?;
    }
    if settings.theme != previous.theme {
        super::apply_theme(&app, &settings.theme);
    }
    if previous.auto_update && !settings.auto_update {
        updates::cancel(&app);
    }
    Ok(store.update(&app, |data| data.settings = settings))
}

#[tauri::command]
pub async fn clear_history(app: AppHandle, store: State<'_, Store>) -> Result<Data, String> {
    Ok(store.update(&app, |data| data.history.clear()))
}

#[tauri::command]
pub async fn copy(clipboard: State<'_, Clipboard>, text: String) -> Result<(), String> {
    clipboard.copy(&text)
}

/// Released while you record a new shortcut, so the old one doesn't fire.
#[tauri::command]
pub async fn pause_shortcut(app: AppHandle, store: State<'_, Store>, paused: bool) -> Result<(), String> {
    if paused {
        picker::clear_shortcut(&app)
    } else {
        picker::set_shortcut(&app, &store.get().settings.shortcut)
    }
}

// ------------------------------------------------------------
// The picker
// ------------------------------------------------------------

#[tauri::command]
pub async fn start_pick(app: AppHandle) {
    picker::start_from_main(&app);
}

/// The screenshot for the picker, handed over once to free memory.
/// Empty when nothing is waiting.
#[tauri::command]
pub async fn frame(frame: State<'_, Frame>) -> Result<Response, String> {
    Ok(Response::new(frame.data.lock().unwrap().take().unwrap_or_default()))
}

/// The picker has drawn the screenshot, so it can show without a flash.
/// Also called again whenever it notices it lost keyboard focus.
#[tauri::command]
pub async fn ready(window: WebviewWindow) {
    let _ = window.show();
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
    // The window being in front isn't enough; keys go to the focused page.
    let webview: &tauri::Webview = window.as_ref();
    let _ = webview.set_focus();
}

/// Arrow keys in the picker: puts the mouse cursor on screen pixel `x, y` of
/// the picker. Not on Linux: Wayland doesn't let apps move the cursor, and
/// XPicker draws its own crosshair there anyway.
#[tauri::command]
pub async fn move_cursor(window: WebviewWindow, x: i32, y: i32) {
    let _ = window.set_cursor_position(tauri::PhysicalPosition::new(x, y));
}

#[tauri::command]
pub async fn pick(app: AppHandle, value: String, hex: String, format: usize) -> Result<(), String> {
    picker::finish(&app, &value, &hex, format)
}

/// Cancel in the picker.
#[tauri::command]
pub async fn dismiss(app: AppHandle) {
    picker::cancel(&app);
}

/// "Open XPicker" in the picker.
#[tauri::command]
pub async fn open_from_picker(app: AppHandle) {
    picker::open_app(&app);
}

// ------------------------------------------------------------
// Updates
// ------------------------------------------------------------

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<String>, String> {
    updates::check(&app).await
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    updates::install(&app).await
}

/// Version of a downloaded update waiting to install, if any.
#[tauri::command]
pub async fn pending_update(app: AppHandle) -> Option<String> {
    updates::ready_version(&app)
}

// ------------------------------------------------------------
// Links and folders
// ------------------------------------------------------------

/// Footer links. Only these addresses can be opened from the window.
const LINKS: [(&str, &str); 3] = [
    ("github", "https://github.com/QuintixLabs/XPicker"),
    ("website", "https://xpickerapp.netlify.app"),
    ("kofi", "https://ko-fi.com/fr0stiwnl"),
];

#[tauri::command]
pub async fn open_link(app: AppHandle, name: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let (_, url) = LINKS.iter().find(|(key, _)| *key == name).ok_or("unknown link")?;
    app.opener().open_url(*url, None::<&str>).map_err(|e| e.to_string())
}

/// "Open settings folder": shows xpicker.json in the file manager.
#[tauri::command]
pub async fn open_settings_folder(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let dir = settings::settings_dir(&app);
    let file = dir.join("xpicker.json");
    if file.exists() {
        app.opener().reveal_item_in_dir(&file).map_err(|e| e.to_string())
    } else {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        app.opener().open_path(dir.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())
    }
}
