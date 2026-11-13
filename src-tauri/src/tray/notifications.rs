// ============================================================
// tray/notifications.rs
// XPicker's notifications
// ============================================================

use std::{path::PathBuf, time::SystemTime};
use tauri::{AppHandle, Manager};

use crate::settings::Store;

/// How long a "copied" notification is kept before it removes itself.
const COPIED_LIFETIME_SECS: i64 = 6;

/// Sent before returning, since in quick pick the app quits right after.
pub fn copied(app: &AppHandle, value: &str, hex: &str) {
    let image = swatch(app, hex);
    if let Err(err) = platform::show(&app_id(app), value, "Copied to clipboard", image, true) {
        eprintln!("XPicker: couldn't show a notification: {err}");
    }
}

/// Update news. These stay in the notification center. Like everything here,
/// they follow the Notifications setting.
pub fn info(app: &AppHandle, title: &str, body: &str) {
    if app.state::<Store>().get().settings.notify {
        show(app, title, body, None, false);
    }
}

/// A small square of the color, saved once per color in XPicker's cache folder.
/// Only the colors XPicker still keeps (the last 20, whatever Recent colors
/// shows) keep theirs; the rest are removed whenever a new one is made.
fn swatch(app: &AppHandle, hex: &str) -> Option<PathBuf> {
    let n = u32::from_str_radix(hex, 16).ok()?;
    let dir = app.path().cache_dir().ok()?.join("XPicker").join("swatches");
    let path = dir.join(format!("{hex}.png"));
    if !path.exists() {
        std::fs::create_dir_all(&dir).ok()?;
        let [_, r, g, b] = n.to_be_bytes();
        xcap::image::RgbaImage::from_pixel(64, 64, xcap::image::Rgba([r, g, b, 255])).save(&path).ok()?;
        let kept: Vec<String> = app.state::<Store>().get().history.into_iter().map(|entry| format!("{}.png", entry.hex)).collect();
        for file in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let name = file.file_name().to_string_lossy().into_owned();
            if name.ends_with(".png") && name != format!("{hex}.png") && !kept.contains(&name) {
                let _ = std::fs::remove_file(file.path());
            }
        }
    }
    Some(path)
}

fn show(app: &AppHandle, title: &str, body: &str, image: Option<PathBuf>, transient: bool) {
    let app_id = app_id(app);
    let (title, body) = (title.to_owned(), body.to_owned());
    // Off the UI thread: talking to the notification service can take a moment.
    std::thread::spawn(move || {
        if let Err(err) = platform::show(&app_id, &title, &body, image, transient) {
            eprintln!("XPicker: couldn't show a notification: {err}");
        }
    });
}

fn app_id(app: &AppHandle) -> String {
    app.config().identifier.clone()
}

/// Windows only shows notifications from apps it knows, and silently drops the
/// rest. The installer's Start menu shortcut introduces XPicker, but that's
/// missing in a dev build, a copy run without installing, or after the shortcut
/// is removed. So XPicker introduces itself at every start: its name and icon,
/// under the current user. The uninstaller removes it again (see hooks.nsh).
pub fn register(app: &AppHandle) {
    #[cfg(windows)]
    if let Err(err) = platform::register(app, &app_id(app)) {
        eprintln!("XPicker: couldn't register for notifications: {err}");
    }
    #[cfg(not(windows))]
    let _ = app;
}

#[cfg(windows)]
mod platform {
    use super::*;
    use windows::{
        core::{Interface, HSTRING},
        Data::Xml::Dom::XmlDocument,
        Foundation::{DateTime, IReference, PropertyValue},
        UI::Notifications::{ToastNotification, ToastNotificationManager},
    };

    pub fn show(app_id: &str, title: &str, body: &str, image: Option<PathBuf>, transient: bool) -> windows::core::Result<()> {
        let image = image
            .map(|p| format!(r#"<image placement="appLogoOverride" src="{}"/>"#, escape(&p.to_string_lossy())))
            .unwrap_or_default();
        let xml = format!(
            r#"<toast><visual><binding template="ToastGeneric"><text>{}</text><text>{}</text>{image}</binding></visual><audio silent="true"/></toast>"#,
            escape(title),
            escape(body),
        );

        let doc = XmlDocument::new()?;
        doc.LoadXml(&HSTRING::from(xml))?;
        let toast = ToastNotification::CreateToastNotification(&doc)?;

        // Once expired, Windows removes it from the notification center.
        if transient {
            let unix = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
            let ticks = (unix + 11_644_473_600 + COPIED_LIFETIME_SECS) * 10_000_000; // 100ns since 1601
            let expiry = PropertyValue::CreateDateTime(DateTime { UniversalTime: ticks })?.cast::<IReference<DateTime>>()?;
            toast.SetExpirationTime(&expiry)?;
        }

        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(app_id))?.Show(&toast)
    }

    fn escape(text: &str) -> String {
        text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
    }

    /// The icon notifications show next to "XPicker".
    const ICON: &[u8] = include_bytes!("../../icons/app/128x128.png");

    pub fn register(app: &AppHandle, app_id: &str) -> Result<(), String> {
        use windows::{
            core::HSTRING,
            Win32::System::Registry::{RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ},
        };

        let icon = app.path().cache_dir().map_err(|e| e.to_string())?.join("XPicker").join("icon.png");
        if !icon.exists() {
            std::fs::create_dir_all(icon.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(&icon, ICON).map_err(|e| e.to_string())?;
        }

        let key = HSTRING::from(format!("Software\\Classes\\AppUserModelId\\{app_id}"));
        for (name, value) in [("DisplayName", app.package_info().name.clone()), ("IconUri", icon.to_string_lossy().into_owned())] {
            let data: Vec<u16> = value.encode_utf16().chain([0]).collect();
            let result = unsafe {
                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    &key,
                    &HSTRING::from(name),
                    REG_SZ.0,
                    Some(data.as_ptr().cast()),
                    (data.len() * 2) as u32,
                )
            };
            result.ok().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    use super::*;

    pub fn show(app_id: &str, title: &str, body: &str, image: Option<PathBuf>, transient: bool) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let _ = notify_rust::set_application(app_id);
        }
        let _ = app_id;

        let mut notification = notify_rust::Notification::new();
        notification.appname("XPicker").summary(title).body(body);

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            if let Some(path) = &image {
                notification.image_path(&path.to_string_lossy());
            }
            // Shown, but not kept in the notification history. Not on GNOME: it
            // drops those if they come while it still thinks the picker is up.
            let gnome = std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|d| d.to_lowercase().contains("gnome"));
            if transient && !gnome {
                notification.hint(notify_rust::Hint::Transient(true));
            }
        }
        let _ = (image, transient, COPIED_LIFETIME_SECS, SystemTime::now());

        let shown = notification.show().map(|_| ()).map_err(|e| e.to_string());
        if std::env::var_os("XPICKER_DEBUG").is_some() {
            eprintln!("XPicker: notification {title}: {:?}", shown);
        }
        shown
    }
}
