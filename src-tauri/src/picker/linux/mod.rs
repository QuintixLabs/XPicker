// ============================================================
// picker/linux/mod.rs
// Linux: on X11, xcap and Tauri already do everything. On Wayland
// apps can't see the cursor or the screen, so each desktop is asked
// in its own way (wayland/).
// ============================================================

mod wayland;

use tauri::{AppHandle, Monitor, PhysicalPosition};

fn on_wayland() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some() || std::env::var("XDG_SESSION_TYPE").is_ok_and(|t| t == "wayland")
}

/// Where the cursor is, from the desktop. None: ask Tauri as usual.
pub fn cursor(app: &AppHandle) -> Option<PhysicalPosition<f64>> {
    if !on_wayland() {
        return None;
    }
    let (x, y) = wayland::cursor()?;
    // The desktop counts in its own units (logical pixels); Tauri in each
    // screen's real pixels.
    app.available_monitors().ok()?.into_iter().find_map(|monitor| {
        let scale = monitor.scale_factor();
        let (left, top) = (monitor.position().x as f64 / scale, monitor.position().y as f64 / scale);
        let (width, height) = (monitor.size().width as f64 / scale, monitor.size().height as f64 / scale);
        let inside = x >= left && y >= top && x < left + width && y < top + height;
        inside.then(|| PhysicalPosition::new(monitor.position().x as f64 + (x - left) * scale, monitor.position().y as f64 + (y - top) * scale))
    })
}

/// A screenshot of `monitor` as (width, height, RGBA pixels). None: use xcap.
pub fn capture(monitor: &Monitor) -> Option<Result<(u32, u32, Vec<u8>), String>> {
    if !on_wayland() {
        return None;
    }
    let scale = monitor.scale_factor();
    let left = (monitor.position().x as f64 / scale).round() as i32;
    let top = (monitor.position().y as f64 / scale).round() as i32;
    let width = (monitor.size().width as f64 / scale).round() as u32;
    let height = (monitor.size().height as f64 / scale).round() as u32;
    Some(wayland::capture(left, top, width, height))
}

/// When XPicker starts.
pub fn start() {
    if on_wayland() {
        wayland::start();
    }
}

/// For Settings: what's left to set up on this desktop ("" for nothing,
/// "needed", or "login" once only logging out and back in is left).
pub fn setup_state() -> &'static str {
    if on_wayland() { wayland::setup_state() } else { "" }
}

pub fn set_up() -> Result<&'static str, String> {
    if on_wayland() { wayland::set_up() } else { Ok("") }
}

/// The picker was just shown: some desktops need help covering the screen.
pub fn cover_screen() {
    if on_wayland() {
        wayland::cover_screen();
    }
}

/// The shortcut, for desktops that don't pass keys to Tauri's own shortcuts.
pub fn set_shortcut(app: &AppHandle, shortcut: &str) {
    if on_wayland() {
        wayland::set_shortcut(app, shortcut);
    }
}
