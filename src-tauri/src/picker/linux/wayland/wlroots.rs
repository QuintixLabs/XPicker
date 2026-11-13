// ============================================================
// picker/linux/wayland/wlroots.rs
// Screenshots on wlroots desktops (Hyprland, Sway, river).
// ============================================================

use libwayshot_xcap::{
    region::{LogicalRegion, Position, Region, Size},
    WayshotConnection,
};

/// The area in the desktop's own units. None when this desktop can't do it.
pub fn capture(left: i32, top: i32, width: u32, height: u32) -> Option<(u32, u32, Vec<u8>)> {
    let connection = WayshotConnection::new().ok()?;
    let region = LogicalRegion { inner: Region { position: Position { x: left, y: top }, size: Size { width, height } } };
    let image = connection.screenshot(region, false).ok()?;
    Some((image.width(), image.height(), image.to_rgba8().into_vec()))
}
