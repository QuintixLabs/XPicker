//==============================================================
// XPicker — Quintix
// Licensed under the GNU General Public License v3.0
//
// Lives in the tray (or quits when idle if "Close to tray" is off).
// A global shortcut starts a pick; picks are copied, remembered and
// shown in the main window alongside the settings. First launch
// shows a welcome screen.
//==============================================================

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod clipboard;
mod commands;
mod picker;
mod shortcut;
mod store;
mod tray;
mod ui;
mod updates;

use tauri::{Manager, RunEvent};

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| picker::launch(app)))
        .plugin(shortcut::plugin())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(picker::Frame::default())
        .manage(updates::Pending::default())
        .manage(clipboard::Clipboard::default())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle();
            let store = store::Store::load(handle);
            let shortcut = store.get().settings.shortcut;
            app.manage(store);

            tray::create(handle)?;
            if let Err(err) = shortcut::apply(handle, &shortcut) {
                eprintln!("XPicker: couldn't register {shortcut}: {err}");
            }
            picker::launch(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::state,
            commands::frame,
            commands::ready,
            commands::dismiss,
            commands::toast_payload,
            commands::start_pick,
            commands::open_main,
            commands::open_from_picker,
            commands::pick,
            commands::copy,
            commands::save_settings,
            commands::clear_history,
            commands::pause_shortcut,
            commands::check_update,
            commands::install_update,
            commands::open_link,
        ])
        .build(tauri::generate_context!())
        .expect("failed to start XPicker");

    // Last window closed: stay in the tray, or quit if "Close to tray" is off.
    // Quit from the tray menu always quits.
    app.run(|app, event| {
        if let RunEvent::ExitRequested { code: None, api, .. } = event {
            if picker::keep_running(app) {
                api.prevent_exit();
            }
        }
    });
}
