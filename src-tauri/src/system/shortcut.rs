// ============================================================
// shortcut.rs
// The "Start picking" shortcut (Ctrl+Shift+X by default), which
// works from anywhere on your computer.
// ============================================================

use tauri::{plugin::TauriPlugin, AppHandle, Manager, Wry};
use tauri_plugin_global_shortcut::{Builder, GlobalShortcutExt, ShortcutState};

use crate::{app::settings::Store, picker};

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new()
        .with_handler(|app, _shortcut, event| {
            // Does nothing until the welcome screen is finished, so you set
            // up your keys first.
            let welcome = app.state::<Store>().get().settings.show_welcome;
            if event.state() == ShortcutState::Pressed && !welcome {
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
