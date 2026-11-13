// ============================================================
// picker/macos/mod.rs
// macOS only lets XPicker see the screen with the Screen Recording
// permission; without it, a screenshot shows just the wallpaper. So
// each pick checks first, and the window explains what to turn on.
// ============================================================

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

use crate::settings::Store;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

/// A pick was stopped for the permission; the window shows the message once.
static NEEDED: AtomicBool = AtomicBool::new(false);

pub fn allowed() -> bool {
    unsafe { CGPreflightScreenCaptureAccess() }
}

/// One window at a time: the first time, only macOS's own (it shows that just
/// once). After that, only XPicker's message, since macOS won't ask again.
pub fn ask(app: &AppHandle) {
    let store = app.state::<Store>();
    if !store.get().screen_permission_asked {
        store.update(app, |data| data.screen_permission_asked = true);
        unsafe { CGRequestScreenCaptureAccess() };
        return;
    }
    NEEDED.store(true, Ordering::Relaxed);
    let _ = crate::window::open(app);
    let _ = app.emit("screen-permission", ());
}

/// Asked by the window: show the message? Only once per stopped pick.
pub fn take_needed() -> bool {
    NEEDED.swap(false, Ordering::Relaxed)
}

pub fn open_settings() -> Result<(), String> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
        .status()
        .map(|_| ())
        .map_err(|e| format!("Couldn't open System Settings: {e}"))
}
