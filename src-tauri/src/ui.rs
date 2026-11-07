// Windows — picker overlay, copy toast and the main window.
// Overlay and toast get a fresh label each time ("overlay-3"), so a window
// that's still closing can never clash with its replacement.

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};
use tauri::{window::Color, AppHandle, Manager, Monitor, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const MAIN: &str = "main";
pub const OVERLAY: &str = "overlay";
pub const TOAST: &str = "toast";

/// Toast window size in logical px (card + room for its shadow).
const TOAST_SIZE: (f64, f64) = (420.0, 160.0);

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

fn label(kind: &str) -> String {
    format!("{kind}-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed))
}

/// First open window of a kind ("overlay" / "toast").
pub fn find(app: &AppHandle, kind: &str) -> Option<WebviewWindow> {
    let prefix = format!("{kind}-");
    app.webview_windows().into_iter().find(|(l, _)| l.starts_with(&prefix)).map(|(_, w)| w)
}

pub fn close_toasts(app: &AppHandle) {
    while let Some(toast) = find(app, TOAST) {
        if toast.destroy().is_err() {
            break;
        }
    }
}

pub fn create_overlay(app: &AppHandle, monitor: &Monitor) -> tauri::Result<()> {
    let window = WebviewWindowBuilder::new(app, label(OVERLAY), WebviewUrl::App("overlay.html".into()))
        .title("XPicker")
        .decorations(false)
        .resizable(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()?;

    window.set_position(*monitor.position())?;
    window.set_size(*monitor.size())?;
    Ok(())
}

/// Created invisible and click-through when picking starts, so it can appear
/// the instant a color is picked (creating a webview then would add a delay).
pub fn create_toast(app: &AppHandle, monitor: &Monitor) -> tauri::Result<()> {
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let right = (area.position.x as f64 + area.size.width as f64) / scale;
    let bottom = (area.position.y as f64 + area.size.height as f64) / scale;

    let window = WebviewWindowBuilder::new(app, label(TOAST), WebviewUrl::App("toast.html".into()))
        .title("XPicker")
        .inner_size(TOAST_SIZE.0, TOAST_SIZE.1)
        .position(right - TOAST_SIZE.0, bottom - TOAST_SIZE.1)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .build()?;

    window.set_ignore_cursor_events(true)?;
    Ok(())
}

/// For callbacks running on the UI thread (tray, single-instance).
pub fn open_main_async(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || {
        let _ = open_main(&app);
    });
}

pub fn open_main(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN) {
        window.unminimize()?;
        window.show()?;
        return window.set_focus();
    }

    WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("index.html".into()))
        .title("XPicker")
        .inner_size(760.0, 600.0)
        .resizable(false)
        .maximizable(false)
        .center()
        .theme(Some(Theme::Dark))
        .background_color(Color(11, 11, 12, 255))
        .build()?;
    Ok(())
}
