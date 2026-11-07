//==============================================================
// XPicker — Quintix
// Licensed under the GNU General Public License v3.0
//
// Lives in the tray (or quits when idle if "Close to tray" is off).
// A global shortcut starts a pick; picks are copied, remembered and
// shown in the main window alongside the settings. First launch
// shows a welcome screen. Updates itself quietly when not in use.
//==============================================================

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod clipboard;
mod commands;
mod notify;
mod picker;
mod shortcut;
mod startup;
mod store;
mod tray;
mod ui;
mod updates;

use std::time::Duration;
use tauri::{AppHandle, Manager, RunEvent};

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| picker::launch(app)))
        .plugin(shortcut::plugin())
        .plugin(startup::plugin())
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
            open_on_start(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::state,
            commands::frame,
            commands::ready,
            commands::dismiss,
            commands::start_pick,
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

    app.run(|app, event| {
        if let RunEvent::ExitRequested { code, api, .. } = event {
            // Last window closed: stay in the tray, unless "Close to tray" is off.
            // Quit from the tray menu (code is set) always quits.
            if code.is_none() && picker::keep_running(app) {
                api.prevent_exit();
            } else {
                updates::install_on_exit(app);
            }
        }
    });
}

/// What this start should show, then start the update checker.
fn open_on_start(app: &AppHandle) {
    let at_login = startup::launched_at_login();
    match updates::just_updated(app).map(|notice| notice.mode) {
        // Restarted by a quiet update: back to the tray, nothing pops up.
        Some(mode) if mode == "background" => {}
        Some(mode) if mode == "manual" => ui::open_main_async(app),
        // Login start: quietly in the tray.
        _ if at_login => {}
        _ => picker::launch(app),
    }
    // A login start waits a minute so it doesn't slow down logging in. Quick pick
    // checks right away, since the app only lives for one pick.
    let settings = app.state::<store::Store>().get().settings;
    let quick_pick = settings.pick_on_launch && !settings.close_to_tray;
    let delay = if at_login { 60 } else if quick_pick { 0 } else { 10 };
    updates::start_auto(app, Duration::from_secs(delay));
}
