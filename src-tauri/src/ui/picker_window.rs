// ============================================================
// picker_window.rs
// The full-screen picker window. With "Instant picker" on, it's
// made once and kept hidden so it opens right away; otherwise it's
// made for each pick and closed after.
// ============================================================

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};
use tauri::{AppHandle, Emitter, Manager, Monitor, WebviewWindow};

use super::builder;
use crate::app::settings::Store;

/// Each picker window gets a fresh label ("picker-3"), so one that's still
/// closing can never clash with the next one.
const LABEL: &str = "picker";
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// The picker window, open or kept ready.
pub fn find(app: &AppHandle) -> Option<WebviewWindow> {
    let prefix = format!("{LABEL}-");
    app.webview_windows().into_iter().find(|(label, _)| label.starts_with(&prefix)).map(|(_, w)| w)
}

/// Someone is picking right now.
pub fn is_open(app: &AppHandle) -> bool {
    find(app).is_some_and(|w| w.is_visible().unwrap_or(false))
}

/// Instant picker only makes sense while XPicker stays in the tray.
fn keep_ready(app: &AppHandle) -> bool {
    let settings = app.state::<Store>().get().settings;
    settings.instant_picker && settings.close_to_tray
}

fn create(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let label = format!("{LABEL}-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
    builder(app, label, "picker.html")
        .title("XPicker")
        .decorations(false)
        .resizable(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
}

// ------------------------------------------------------------
// Opening and closing
// ------------------------------------------------------------

/// Covers `monitor` with the picker. The screenshot is already waiting; the
/// picker draws it, then shows itself (see the "ready" command).
pub fn open(app: &AppHandle, monitor: &Monitor) -> tauri::Result<()> {
    let window = match find(app) {
        Some(window) => window,
        None => create(app)?,
    };
    window.set_position(*monitor.position())?;
    window.set_size(*monitor.size())?;
    // A ready window is already loaded, so tell it a pick has started. A new
    // one fetches the screenshot by itself as soon as it loads.
    let _ = app.emit_to(window.label(), "picker-open", ());
    Ok(())
}

/// After a pick or cancel: hide it to reuse next time, or close it.
pub fn close(app: &AppHandle) {
    if let Some(window) = find(app) {
        if keep_ready(app) {
            let _ = window.hide();
        } else {
            let _ = window.destroy();
        }
    }
}

/// Makes or removes the hidden, ready window to match the settings.
pub fn sync(app: &AppHandle) {
    match find(app) {
        None if keep_ready(app) => {
            let _ = create(app);
        }
        Some(window) if !keep_ready(app) && !window.is_visible().unwrap_or(false) => {
            let _ = window.destroy();
        }
        _ => {}
    }
}

/// For the UI thread (startup): windows are made off it.
pub fn sync_async(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || sync(&app));
}
