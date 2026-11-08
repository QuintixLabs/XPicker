// ============================================================
// settings.rs
// The settings file: your settings and recent colors, saved in
//   Windows  %APPDATA%\XPicker\xpicker.json
//   macOS    ~/Library/Application Support/XPicker/xpicker.json
//   Linux    ~/.config/XPicker/xpicker.json
// To see the welcome screen again: exit XPicker, set "showWelcome": true.
// ============================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf, sync::Mutex, time::SystemTime};
use tauri::{AppHandle, Emitter, Manager};

use crate::system::tray;

const FILE: &str = "xpicker.json";

/// The largest "Recent colors" option in Settings.
const MAX_HISTORY: usize = 20;

/// Where settings live: a plain "XPicker" folder in the system's config folder.
pub fn settings_dir(app: &AppHandle) -> PathBuf {
    app.path().config_dir().unwrap_or_else(|_| PathBuf::from(".")).join("XPicker")
}

// ------------------------------------------------------------
// What's saved
// ------------------------------------------------------------

/// Bindings are "Mouse:<button>" or "Key:<KeyboardEvent.code>".
/// The global shortcut uses the plugin's syntax, e.g. "Ctrl+Shift+KeyX".
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub shortcut: String,
    pub pick: String,
    pub cycle: String,
    pub cancel: String,
    pub open: String,
    /// "system", "light" or "dark".
    pub theme: String,
    /// All of XPicker's notifications.
    pub notify: bool,
    pub history_size: usize,
    pub format: usize,
    /// Keep the picker window ready so it opens right away.
    pub instant_picker: bool,
    pub start_with_computer: bool,
    pub pick_on_launch: bool,
    /// Off: quit once nothing is open (with pick_on_launch, that's "quick pick").
    pub close_to_tray: bool,
    pub auto_update: bool,
    /// Show the welcome screen on the next start (true until it's been gone through).
    pub show_welcome: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: "Ctrl+Shift+KeyX".into(),
            pick: "Mouse:2".into(),
            cycle: "Key:Control".into(),
            cancel: "Key:Escape".into(),
            open: "Key:Tab".into(),
            theme: "system".into(),
            notify: true,
            history_size: 10,
            format: 0,
            instant_picker: true,
            start_with_computer: false,
            pick_on_launch: false,
            close_to_tray: true,
            auto_update: true,
            show_welcome: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub hex: String,
    pub at: u64,
}

/// Left behind right before an update installs, read on the next start.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotice {
    pub version: String,
    /// "background" (restart quietly into the tray), "exit" (installed while
    /// closing; the next start is a normal one) or "manual" (reopen the window).
    pub mode: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Data {
    pub settings: Settings,
    pub history: Vec<Entry>,
    /// Unix ms of the last automatic update check.
    pub last_update_check: u64,
    pub update_notice: Option<UpdateNotice>,
}

// ------------------------------------------------------------
// Reading and saving
// ------------------------------------------------------------

pub struct Store {
    path: PathBuf,
    data: Mutex<Data>,
}

impl Store {
    pub fn load(app: &AppHandle) -> Self {
        let path = settings_dir(app).join(FILE);
        move_from_old_folder(app, &path);
        let data = fs::read_to_string(&path)
            .ok()
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .map(upgrade)
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or_default();
        Self { path, data: Mutex::new(data) }
    }

    pub fn get(&self) -> Data {
        self.data.lock().unwrap().clone()
    }

    /// Changes, saves to disk, then tells every window and the tray menu.
    pub fn update(&self, app: &AppHandle, change: impl FnOnce(&mut Data)) -> Data {
        let data = {
            let mut data = self.data.lock().unwrap();
            change(&mut data);
            // Keep the most "Recent colors" can ask for; the window shows only
            // history_size. So switching 10 → 5 → 10 never loses colors.
            data.history.truncate(MAX_HISTORY);

            if let Some(dir) = self.path.parent() {
                let _ = fs::create_dir_all(dir);
            }
            if let Ok(json) = serde_json::to_string_pretty(&*data) {
                let _ = fs::write(&self.path, json);
            }
            data.clone()
        }; // lock released: the tray reads the settings while rebuilding

        let _ = app.emit("state", &data);
        tray::refresh(app);
        data
    }
}

/// Moves `hex` to the top of the recent colors.
pub fn remember(data: &mut Data, hex: &str) {
    data.history.retain(|entry| entry.hex != hex);
    data.history.insert(0, Entry { hex: hex.into(), at: now_ms() });
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or_default()
}

// ------------------------------------------------------------
// Older versions
// ------------------------------------------------------------

/// Versions before 1.0.9 kept settings in a folder named after the app ID
/// (com.quintixlabs.xpicker). Move the file over once, then remove the old folder.
fn move_from_old_folder(app: &AppHandle, path: &PathBuf) {
    let Ok(old_dir) = app.path().app_config_dir() else { return };
    let old = old_dir.join(FILE);
    if path.exists() || !old.exists() {
        return;
    }
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if fs::rename(&old, path).is_err() && fs::copy(&old, path).is_ok() {
        let _ = fs::remove_file(&old);
    }
    let _ = fs::remove_dir(&old_dir); // only if nothing else is left in it
}

/// Before 1.0.9 the color squares and the window's browser data also sat in
/// app-ID folders. They now live under "XPicker", so remove the old ones.
/// Runs before any window exists, so nothing in them is in use.
pub fn remove_old_cache(app: &AppHandle) {
    let id = app.config().identifier.clone();
    let mut old = vec![app.path().app_cache_dir()];
    if !cfg!(target_os = "macos") {
        // On macOS this is the old settings folder, handled by move_from_old_folder.
        old.push(app.path().app_local_data_dir());
    }
    for dir in old.into_iter().flatten() {
        if dir.file_name().is_some_and(|name| name == id.as_str()) {
            let _ = fs::remove_dir_all(dir);
        }
    }
}

/// Older files said "onboarded": true once the welcome screen was done;
/// that's now "showWelcome": false.
fn upgrade(mut value: Value) -> Value {
    if let Some(settings) = value.get_mut("settings").and_then(Value::as_object_mut) {
        if let Some(onboarded) = settings.remove("onboarded") {
            settings.entry("showWelcome").or_insert(Value::Bool(!onboarded.as_bool().unwrap_or(false)));
        }
    }
    value
}
