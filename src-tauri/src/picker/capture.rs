// ============================================================
// capture.rs
// Takes a screenshot of the screen the mouse is on.
// ============================================================

use tauri::{Monitor, PhysicalPosition};
use xcap::Monitor as Screen;

/// What the picker receives: five little-endian numbers
/// [width, height, cursor x, cursor y, screen scale × 1000], then the pixels (RGBA).
/// The scale lets the picker size itself right away, even while its window is
/// still being moved to this screen.
pub fn grab(monitor: &Monitor, cursor: PhysicalPosition<f64>) -> Result<Vec<u8>, String> {
    let pos = monitor.position();
    let size = monitor.size();

    // macOS reports screens in points, everything else in pixels.
    let unit = if cfg!(target_os = "macos") { monitor.scale_factor() } else { 1.0 };
    let center_x = (pos.x as f64 + size.width as f64 / 2.0) / unit;
    let center_y = (pos.y as f64 + size.height as f64 / 2.0) / unit;

    let screen = Screen::from_point(center_x as i32, center_y as i32).map_err(|e| e.to_string())?;
    let image = screen.capture_image().map_err(|e| e.to_string())?;
    let (width, height) = (image.width(), image.height());

    // Cursor relative to the screen, in screenshot pixels.
    let x = (cursor.x - pos.x as f64) * width as f64 / size.width as f64;
    let y = (cursor.y - pos.y as f64) * height as f64 / size.height as f64;
    // Screenshot pixels per window pixel.
    let scale = width as f64 / (size.width as f64 / monitor.scale_factor());

    let mut out = Vec::with_capacity(20 + image.as_raw().len());
    for v in [width as i32, height as i32, x as i32, y as i32, (scale * 1000.0).round() as i32] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(image.as_raw());
    Ok(out)
}
