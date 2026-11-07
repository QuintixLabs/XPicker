// Screen capture — grabs the monitor under the cursor as raw RGBA.

use tauri::{Monitor, PhysicalPosition};
use xcap::Monitor as Screen;

/// Frame layout sent to the overlay:
/// [width, height, cursor_x, cursor_y] as little-endian i32, then RGBA pixels.
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

    // Cursor relative to the monitor, in screenshot pixels.
    let x = (cursor.x - pos.x as f64) * width as f64 / size.width as f64;
    let y = (cursor.y - pos.y as f64) * height as f64 / size.height as f64;

    let mut out = Vec::with_capacity(16 + image.as_raw().len());
    for v in [width as i32, height as i32, x as i32, y as i32] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(image.as_raw());
    Ok(out)
}
