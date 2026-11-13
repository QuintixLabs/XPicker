// ============================================================
// picker/overlay/capture.rs
// Takes a screenshot of the screen the mouse is on.
// ============================================================

use tauri::{Monitor, PhysicalPosition};
use xcap::Monitor as Screen;

/// What the picker receives: nine little-endian numbers, then the pixels (RGBA):
/// [width, height, cursor x, cursor y, screen scale × 1000,
///  taskbar room on the left, top, right, bottom].
/// The scale lets the picker size itself right away, even while its window is
/// still being moved to this screen. The taskbar room keeps the hint bar clear
/// of taskbars and docks. Everything is in screenshot pixels.
pub const HEADER: usize = 36;

pub fn grab(monitor: &Monitor, cursor: PhysicalPosition<f64>) -> Result<Vec<u8>, String> {
    let pos = monitor.position();
    let size = monitor.size();
    let (width, height, pixels) = screenshot(monitor)?;

    // Cursor relative to the screen, in screenshot pixels.
    let x = (cursor.x - pos.x as f64) * width as f64 / size.width as f64;
    let y = (cursor.y - pos.y as f64) * height as f64 / size.height as f64;
    // Screenshot pixels per window pixel.
    let scale = width as f64 / (size.width as f64 / monitor.scale_factor());
    // XPICKER_DEBUG=1 prints what each pick saw, for testing.
    if std::env::var_os("XPICKER_DEBUG").is_some() {
        eprintln!(
            "XPicker: screen at {},{} size {}x{} scale {}; screenshot {width}x{height}; cursor {:.0},{:.0}; picker scale {scale}",
            pos.x,
            pos.y,
            size.width,
            size.height,
            monitor.scale_factor(),
            cursor.x,
            cursor.y
        );
    }

    // The part of the screen not taken by taskbars, as room left on each edge.
    let free = monitor.work_area();
    let (sx, sy) = (width as f64 / size.width as f64, height as f64 / size.height as f64);
    // Some systems don't report it (empty), and no taskbar takes more than a
    // third of the screen; either way, fall back to no room.
    let known = free.size.width > 0 && free.size.height > 0;
    let room = |v: i64, s: f64, max: u32| {
        if known && v > 0 && v < max as i64 / 3 { (v as f64 * s).round() as i32 } else { 0 }
    };
    let left = room(free.position.x as i64 - pos.x as i64, sx, size.width);
    let top = room(free.position.y as i64 - pos.y as i64, sy, size.height);
    let right = room((pos.x as i64 + size.width as i64) - (free.position.x as i64 + free.size.width as i64), sx, size.width);
    let bottom = room((pos.y as i64 + size.height as i64) - (free.position.y as i64 + free.size.height as i64), sy, size.height);

    let mut out = Vec::with_capacity(HEADER + pixels.len());
    for v in [width as i32, height as i32, x as i32, y as i32, (scale * 1000.0).round() as i32, left, top, right, bottom] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&pixels);
    Ok(out)
}

/// (width, height, RGBA pixels). Wayland desktops are asked their own way first.
fn screenshot(monitor: &Monitor) -> Result<(u32, u32, Vec<u8>), String> {
    #[cfg(target_os = "linux")]
    if let Some(shot) = super::super::linux::capture(monitor) {
        return shot;
    }
    let (pos, size) = (monitor.position(), monitor.size());
    // macOS reports screens in points, everything else in pixels.
    let unit = if cfg!(target_os = "macos") { monitor.scale_factor() } else { 1.0 };
    let center_x = (pos.x as f64 + size.width as f64 / 2.0) / unit;
    let center_y = (pos.y as f64 + size.height as f64 / 2.0) / unit;
    let screen = Screen::from_point(center_x as i32, center_y as i32).map_err(|e| e.to_string())?;
    let image = screen.capture_image().map_err(|e| e.to_string())?;
    Ok((image.width(), image.height(), image.into_raw()))
}
