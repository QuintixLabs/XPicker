// IPC commands called from the web UI.
// Anything that touches windows or the global shortcut is async: sync commands
// run on the UI thread, and doing that work there can freeze the app on Windows.

use tauri::{ipc::Response, AppHandle, State, WebviewWindow};

use crate::{
    clipboard::Clipboard,
    picker::{self, Frame},
    shortcut, startup,
    store::{Data, Settings, Store},
    ui, updates,
};

#[tauri::command]
pub fn state(store: State<Store>) -> Data {
    store.get()
}

/// Raw screenshot for the overlay; handed over once to free memory.
#[tauri::command]
pub async fn frame(frame: State<'_, Frame>) -> Result<Response, String> {
    Ok(Response::new(frame.data.lock().unwrap().take().unwrap_or_default()))
}

/// Overlay has drawn the screenshot; safe to show without a flash.
/// Also called again whenever the overlay notices it lost keyboard focus.
#[tauri::command]
pub async fn ready(window: WebviewWindow) {
    let _ = window.show();
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
    // The window being foreground isn't enough; keys go to the focused webview.
    let webview: &tauri::Webview = window.as_ref();
    let _ = webview.set_focus();
}

#[tauri::command]
pub async fn dismiss(app: AppHandle, window: WebviewWindow) {
    let overlay = window.label().starts_with(ui::OVERLAY);
    let _ = window.destroy();
    if overlay {
        picker::closed(&app);
    }
}

#[tauri::command]
pub async fn start_pick(app: AppHandle) {
    picker::start_from_main(&app);
}

#[tauri::command]
pub async fn open_from_picker(app: AppHandle, window: WebviewWindow) {
    picker::open_app(&app, &window);
}

#[tauri::command]
pub async fn pick(app: AppHandle, value: String, hex: String, format: usize) -> Result<(), String> {
    picker::finish(&app, &value, &hex, format)
}

#[tauri::command]
pub async fn copy(clipboard: State<'_, Clipboard>, text: String) -> Result<(), String> {
    clipboard.copy(&text)
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, store: State<'_, Store>, settings: Settings) -> Result<Data, String> {
    let previous = store.get().settings;
    if settings.shortcut != previous.shortcut {
        if let Err(err) = shortcut::apply(&app, &settings.shortcut) {
            let _ = shortcut::apply(&app, &previous.shortcut);
            return Err(format!("That shortcut can't be used ({err})."));
        }
    }
    if settings.start_with_computer != previous.start_with_computer {
        startup::set(&app, settings.start_with_computer)?;
    }
    Ok(store.update(&app, |data| data.settings = settings))
}

#[tauri::command]
pub async fn clear_history(app: AppHandle, store: State<'_, Store>) -> Result<Data, String> {
    Ok(store.update(&app, |data| data.history.clear()))
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<String>, String> {
    updates::check(&app).await
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    updates::install(&app).await
}

/// Footer links. Only these addresses can be opened from the UI.
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

/// Released while the user records a new shortcut, so the old one doesn't fire.
#[tauri::command]
pub async fn pause_shortcut(app: AppHandle, store: State<'_, Store>, paused: bool) -> Result<(), String> {
    if paused {
        shortcut::clear(&app)
    } else {
        shortcut::apply(&app, &store.get().settings.shortcut)
    }
}
