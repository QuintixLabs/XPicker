// Windows — the picker overlay and the main window.
// The overlay gets a fresh label each time ("overlay-3"), so one that's still
// closing can never clash with its replacement.

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};
use tauri::{window::Color, AppHandle, Emitter, Manager, Monitor, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const MAIN: &str = "main";
pub const OVERLAY: &str = "overlay";

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// First open window of a kind ("overlay").
pub fn find(app: &AppHandle, kind: &str) -> Option<WebviewWindow> {
    let prefix = format!("{kind}-");
    app.webview_windows().into_iter().find(|(l, _)| l.starts_with(&prefix)).map(|(_, w)| w)
}

pub fn create_overlay(app: &AppHandle, monitor: &Monitor) -> tauri::Result<()> {
    let label = format!("{OVERLAY}-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App("overlay.html".into()))
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

/// For callbacks running on the UI thread (tray, single-instance).
pub fn open_main_async(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || {
        let _ = open_main(&app);
    });
}

/// Opens the window on a page ("settings").
pub fn open_page_async(app: &AppHandle, page: &'static str) {
    let app = app.clone();
    thread::spawn(move || {
        let _ = open(&app, Some(page));
    });
}

pub fn open_main(app: &AppHandle) -> tauri::Result<()> {
    open(app, None)
}

fn open(app: &AppHandle, page: Option<&str>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN) {
        if let Some(page) = page {
            let _ = window.emit("navigate", page);
        }
        window.unminimize()?;
        window.show()?;
        return window.set_focus();
    }

    let start = page.map(|p| format!("window.__START_PAGE__ = {p:?};")).unwrap_or_default();
    WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("index.html".into()))
        .initialization_script(&start)
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
