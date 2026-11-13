// ============================================================
// main.rs
// Starts XPicker: the tray, the shortcut and the first window.
// ============================================================

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod picker;
mod settings;
mod tray;
mod window;

use std::time::Duration;
use tauri::{AppHandle, Manager, RunEvent};

use tray::{notifications, startup, updates};
use window::commands;

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if args.iter().any(|arg| arg == picker::PICK_FLAG) {
                picker::pick_from_shortcut(app, false);
            } else {
                picker::launch(app, false);
            }
        }))
        .plugin(picker::shortcut_plugin())
        .plugin(startup::plugin())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(picker::Frame::default())
        .manage(updates::Pending::default())
        .manage(picker::colors::clipboard::Clipboard::default())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle();
            let store = settings::Store::load(handle);
            let settings = store.get().settings;
            let shortcut = settings.shortcut.clone();
            app.manage(store);

            notifications::register(handle);
            #[cfg(target_os = "linux")]
            picker::linux::start();
            tray::create(handle)?;
            // Keep the system's "start with computer" in step with the setting, pointing
            // at this copy of XPicker. Not from a dev build, which isn't the installed one.
            if !cfg!(debug_assertions) {
                let _ = startup::set(handle, settings.start_with_computer);
            }
            if let Err(err) = picker::set_shortcut(handle, &shortcut) {
                eprintln!("XPicker: couldn't register {shortcut}: {err}");
            }
            open_on_start(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::state,
            commands::frame,
            commands::ready,
            commands::move_cursor,
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
            commands::pending_update,
            commands::open_link,
            commands::open_settings_folder,
            commands::screen_permission,
            commands::open_screen_settings,
            commands::desktop_setup,
            commands::desktop_set_up,
        ])
        .build(tauri::generate_context!())
        .expect("failed to start XPicker");

    app.run(|app, event| match event {
        // Last window closed: stay in the tray, unless "Close to tray" is off.
        // Exit from the tray menu (code is set) always quits.
        RunEvent::ExitRequested { code, api, .. } => {
            if code.is_none() && picker::keep_running(app) {
                api.prevent_exit();
            } else {
                updates::install_on_exit(app);
            }
        }
        // macOS: opening XPicker while it runs (Finder, Spotlight, Launchpad) doesn't
        // start a second copy; macOS asks the running one to reopen instead. Treat it
        // like opening it again anywhere else, so it can always be reached, even
        // with the tray icon hidden.
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => picker::launch(app, false),
        _ => {}
    });
}

// ------------------------------------------------------------
// First start
// ------------------------------------------------------------

/// What this start should show, then start the update checker.
fn open_on_start(app: &AppHandle) {
    let at_login = startup::launched_at_login();
    match updates::just_updated(app).map(|notice| notice.mode) {
        // Restarted by a quiet update: back to the tray, nothing pops up.
        Some(mode) if mode == "background" => {}
        Some(mode) if mode == "manual" => window::open_async(app),
        // Started by the desktop's shortcut (GNOME): just the picker.
        _ if std::env::args().any(|arg| arg == picker::PICK_FLAG) => picker::pick_from_shortcut(app, true),
        // Login start: quietly in the tray.
        _ if at_login => {}
        _ => picker::launch(app, true),
    }

    // A login start waits a minute so it doesn't slow down logging in. A quick
    // pick (or Close to tray off) checks right away, since the app may only live
    // for one pick.
    let delay = if at_login { 60 } else if !picker::stays_in_tray(app) { 0 } else { 10 };
    updates::start_auto(app, Duration::from_secs(delay));
}
