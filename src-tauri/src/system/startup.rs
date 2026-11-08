// ============================================================
// startup.rs
// "Start with computer": XPicker starts quietly in the tray when
// you log in.
// ============================================================

// Login starts carry the --autostart flag, so they never open a window or the picker.

use tauri::{plugin::TauriPlugin, AppHandle, Wry};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

const FLAG: &str = "--autostart";

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![FLAG]))
}

/// This run was started by the system at login.
pub fn launched_at_login() -> bool {
    std::env::args().any(|arg| arg == FLAG)
}

pub fn set(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let launcher = app.autolaunch();
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    result.map_err(|e| format!("Couldn't change start with computer ({e})."))
}
