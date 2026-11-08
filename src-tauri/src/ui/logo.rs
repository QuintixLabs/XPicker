// ============================================================
// logo.rs
// Picks the logo that stays visible: the white-X logo on dark
// backgrounds, the dark-X one on light. Used by the title bar and
// the tray.
// ============================================================

// Fixed icons (installer, Start menu, notifications) can't swap, so they use
// the outlined logo from src-tauri/icons instead.

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{image::Image, Theme, WebviewWindow};

/// The last light/dark seen by taskbar_is_light(), kept current by the tray.
static LIGHT: AtomicBool = AtomicBool::new(false);

pub fn remember_light(light: bool) {
    LIGHT.store(light, Ordering::Relaxed);
}

/// Linux: whether the desktop is light, as last checked. The windows can't
/// always see it change themselves there (see ui/mod.rs).
pub fn system_is_light() -> bool {
    LIGHT.load(Ordering::Relaxed)
}

const WINDOW: &[u8] = include_bytes!("../../icons/runtime/logo.png");
const WINDOW_LIGHT: &[u8] = include_bytes!("../../icons/runtime/logo-light.png");
const TRAY: &[u8] = include_bytes!("../../icons/runtime/tray.png");
const TRAY_LIGHT: &[u8] = include_bytes!("../../icons/runtime/tray-light.png");

pub fn tray_icon(light_background: bool) -> Option<Image<'static>> {
    Image::from_bytes(if light_background { TRAY_LIGHT } else { TRAY }).ok()
}

/// Title bar icon, matched to the title bar's color. On Windows and macOS that
/// follows XPicker's theme; on Linux the desktop draws the title bar with its
/// own theme, which is what window.theme() reports there.
pub fn update_window(window: &WebviewWindow) {
    let light = window.theme().map(|t| t == Theme::Light).unwrap_or(false);
    if let Ok(icon) = Image::from_bytes(if light { WINDOW_LIGHT } else { WINDOW }) {
        let _ = window.set_icon(icon);
    }
}

/// Is the taskbar / menu bar / panel light? On Windows this is its own setting,
/// separate from the apps' light/dark mode.
#[cfg(windows)]
pub fn taskbar_is_light() -> bool {
    use windows::{
        core::w,
        Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD},
    };
    let mut value: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("SystemUsesLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut _),
            Some(&mut size),
        )
    };
    result.is_ok() && value == 1
}

#[cfg(not(windows))]
pub fn taskbar_is_light() -> bool {
    matches!(dark_light::detect(), Ok(dark_light::Mode::Light))
}
