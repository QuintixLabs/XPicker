// Global shortcut — starts a pick from any app.

use tauri::{plugin::TauriPlugin, AppHandle, Wry};
use tauri_plugin_global_shortcut::{Builder, GlobalShortcutExt, ShortcutState};

use crate::picker;

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                picker::start(app);
            }
        })
        .build()
}

pub fn apply(app: &AppHandle, shortcut: &str) -> Result<(), String> {
    clear(app)?;
    if shortcut.is_empty() {
        return Ok(());
    }
    app.global_shortcut().register(shortcut).map_err(|e| e.to_string())
}

pub fn clear(app: &AppHandle) -> Result<(), String> {
    app.global_shortcut().unregister_all().map_err(|e| e.to_string())
}
