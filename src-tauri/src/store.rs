// Persistent state — settings + recent colors, saved as JSON in the config dir.

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex, time::SystemTime};
use tauri::{AppHandle, Emitter, Manager};

/// The largest "Remember" option in Settings.
const MAX_HISTORY: usize = 20;

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
    /// System notification after copying a color.
    pub notify: bool,
    pub history_size: usize,
    pub format: usize,
    pub start_with_computer: bool,
    pub pick_on_launch: bool,
    /// Off: quit once nothing is open (with pick_on_launch, that's "quick pick").
    pub close_to_tray: bool,
    pub auto_update: bool,
    pub onboarded: bool,
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
            start_with_computer: false,
            pick_on_launch: false,
            close_to_tray: true,
            auto_update: true,
            onboarded: false,
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

pub struct Store {
    path: PathBuf,
    data: Mutex<Data>,
}

impl Store {
    pub fn load(app: &AppHandle) -> Self {
        let dir = app.path().app_config_dir().unwrap_or_else(|_| PathBuf::from("."));
        let path = dir.join("xpicker.json");
        let data = fs::read_to_string(&path)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();
        Self { path, data: Mutex::new(data) }
    }

    pub fn get(&self) -> Data {
        self.data.lock().unwrap().clone()
    }

    /// Mutates, saves to disk, then tells every window and the tray menu.
    pub fn update(&self, app: &AppHandle, change: impl FnOnce(&mut Data)) -> Data {
        let data = {
            let mut data = self.data.lock().unwrap();
            change(&mut data);
            // Keep the most "Remember" can ask for; the window shows only history_size.
            // So switching 10 → 5 → 10 never loses colors.
            data.history.truncate(MAX_HISTORY);

            if let Some(dir) = self.path.parent() {
                let _ = fs::create_dir_all(dir);
            }
            if let Ok(json) = serde_json::to_string_pretty(&*data) {
                let _ = fs::write(&self.path, json);
            }
            data.clone()
        }; // lock released: the tray reads the store while rebuilding

        let _ = app.emit("state", &data);
        crate::tray::refresh(app);
        data
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or_default()
}

/// Moves `hex` to the top of the history.
pub fn remember(data: &mut Data, hex: &str) {
    data.history.retain(|entry| entry.hex != hex);
    data.history.insert(0, Entry { hex: hex.into(), at: now_ms() });
}
