// Persistent state — settings + recent colors, saved as JSON in the config dir.

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex, time::SystemTime};
use tauri::{AppHandle, Emitter, Manager};

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
    pub toast_ms: u32,
    pub history_size: usize,
    pub format: usize,
    pub pick_on_launch: bool,
    /// Off: quit once nothing is open (with pick_on_launch, that's "quick pick").
    pub close_to_tray: bool,
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
            toast_ms: 3000,
            history_size: 10,
            format: 0,
            pick_on_launch: false,
            close_to_tray: true,
            onboarded: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub hex: String,
    pub at: u64,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub settings: Settings,
    pub history: Vec<Entry>,
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

    /// Mutates, saves to disk and notifies every window.
    pub fn update(&self, app: &AppHandle, change: impl FnOnce(&mut Data)) -> Data {
        let mut data = self.data.lock().unwrap();
        change(&mut data);
        let size = data.settings.history_size;
        data.history.truncate(size);

        if let Some(dir) = self.path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(&*data) {
            let _ = fs::write(&self.path, json);
        }
        let _ = app.emit("state", &*data);
        data.clone()
    }
}

/// Moves `hex` to the top of the history.
pub fn remember(data: &mut Data, hex: &str) {
    let at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default();
    data.history.retain(|entry| entry.hex != hex);
    data.history.insert(0, Entry { hex: hex.into(), at });
}
