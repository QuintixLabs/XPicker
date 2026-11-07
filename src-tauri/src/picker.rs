// Picking flow — screenshot, overlay, then copy + remember + toast.
//
// Windows are only ever created/destroyed from worker threads: doing it on the
// UI thread inside a hotkey/tray callback deadlocks WebView2 on Windows.

use std::{
    error::Error,
    sync::{atomic::{AtomicBool, Ordering}, Mutex},
    thread,
    time::Duration,
};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::{capture, clipboard::Clipboard, store::{self, Store}, ui};

#[derive(Default)]
pub struct Frame {
    /// Screenshot waiting to be fetched by the overlay.
    pub data: Mutex<Option<Vec<u8>>>,
    /// Picked color waiting for the toast, in case it loads after the pick.
    pub toast: Mutex<Option<Value>>,
    /// The main window was hidden for this pick and should come back after.
    restore_main: AtomicBool,
    /// A pick is being set up; ignore repeated triggers.
    starting: AtomicBool,
}

/// App opened (first launch or relaunch): picker or main window, per settings.
pub fn launch(app: &AppHandle) {
    let settings = app.state::<Store>().get().settings;
    if settings.onboarded && settings.pick_on_launch {
        start(app);
    } else {
        ui::open_main_async(app);
    }
}

/// Safe to call from any thread or callback.
pub fn start(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || run(&app, false));
}

/// From the main window's button: hide it first so it isn't in the screenshot.
pub fn start_from_main(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || run(&app, true));
}

/// Asked when the last window closes. Stays alive in the tray unless
/// "Close to tray" is off; never quits halfway through opening the picker.
pub fn keep_running(app: &AppHandle) -> bool {
    app.state::<Frame>().starting.load(Ordering::Acquire) || app.state::<Store>().get().settings.close_to_tray
}

/// "Open XPicker" pressed in the picker: show the app, then close the picker.
pub fn open_app(app: &AppHandle, overlay: &tauri::WebviewWindow) {
    let _ = ui::open_main(app); // first, so there's never a moment with no window
    let _ = overlay.destroy();
    cancelled(app);
}

/// Picker closed without a pick.
pub fn cancelled(app: &AppHandle) {
    ui::close_toasts(app);
    closed(app);
}

/// Overlay is gone (picked or cancelled).
fn closed(app: &AppHandle) {
    if app.state::<Frame>().restore_main.swap(false, Ordering::Relaxed) {
        if let Some(main) = app.get_webview_window(ui::MAIN) {
            let _ = main.show();
        }
    }
}

fn run(app: &AppHandle, hide_main: bool) {
    let state = app.state::<Frame>();
    if state.starting.swap(true, Ordering::AcqRel) {
        return;
    }
    if ui::find(app, ui::OVERLAY).is_none() {
        ui::close_toasts(app);

        if hide_main {
            if let Some(main) = app.get_webview_window(ui::MAIN) {
                if main.is_visible().unwrap_or(false) {
                    let _ = main.hide();
                    state.restore_main.store(true, Ordering::Relaxed);
                    thread::sleep(Duration::from_millis(180));
                }
            }
        }

        if let Err(err) = open(app) {
            eprintln!("XPicker: couldn't start picking: {err}");
            cancelled(app);
        }
    }
    state.starting.store(false, Ordering::Release);
}

fn open(app: &AppHandle) -> Result<(), Box<dyn Error>> {
    let cursor = app.cursor_position()?;
    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)?
        .or(app.primary_monitor()?)
        .ok_or("no monitor found")?;

    let frame = capture::grab(&monitor, cursor)?;
    let state = app.state::<Frame>();
    *state.data.lock().unwrap() = Some(frame);
    *state.toast.lock().unwrap() = None;

    if app.state::<Store>().get().settings.toast_ms > 0 {
        ui::create_toast(app, &monitor)?;
    }
    ui::create_overlay(app, &monitor)?;
    Ok(())
}

/// Runs on a worker thread (async command).
pub fn finish(app: &AppHandle, value: &str, hex: &str, format: usize, name: &str) -> Result<(), String> {
    app.state::<Clipboard>().copy(value)?;
    let data = app.state::<Store>().update(app, |data| {
        store::remember(data, hex);
        data.settings.format = format;
    });

    if let Some(overlay) = ui::find(app, ui::OVERLAY) {
        let _ = overlay.destroy();
    }
    closed(app);

    if let Some(toast) = ui::find(app, ui::TOAST) {
        let payload = json!({ "value": value, "hex": hex, "name": name, "ms": data.settings.toast_ms });
        *app.state::<Frame>().toast.lock().unwrap() = Some(payload.clone());
        let _ = toast.set_ignore_cursor_events(false);
        let _ = app.emit_to(toast.label(), "toast", payload);
    }
    Ok(())
}
