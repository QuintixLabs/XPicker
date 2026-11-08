// ============================================================
// notifications.rs
// XPicker's notifications ("Copied to clipboard", "XPicker was
// updated"), shown as real system notifications.
// ============================================================

// "Copied" ones show the picked color as their image (Windows, most Linux) and
// remove themselves from the notification center so they don't pile up.

use std::{path::PathBuf, time::SystemTime};
use tauri::{AppHandle, Manager};

use crate::app::settings::Store;

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
fn swatch(app: &AppHandle, hex: &str) -> Option<PathBuf> {
    let n = u32::from_str_radix(hex, 16).ok()?;
    let dir = app.path().cache_dir().ok()?.join("XPicker").join("swatches");
    let path = dir.join(format!("{hex}.png"));
    if !path.exists() {
        std::fs::create_dir_all(&dir).ok()?;
        let [_, r, g, b] = n.to_be_bytes();
        xcap::image::RgbaImage::from_pixel(64, 64, xcap::image::Rgba([r, g, b, 255])).save(&path).ok()?;
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

/// Windows only shows notifications for apps it knows. Installed XPicker is
/// registered by the installer; a dev build borrows PowerShell's id instead.
fn app_id(app: &AppHandle) -> String {
    if cfg!(debug_assertions) {
        "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe".into()
    } else {
        app.config().identifier.clone()
    }
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
            if transient {
                // Shown, but not kept in the notification history.
                notification.hint(notify_rust::Hint::Transient(true));
            }
        }
        let _ = (image, transient, COPIED_LIFETIME_SECS, SystemTime::now());

        notification.show().map(|_| ()).map_err(|e| e.to_string())
    }
}
