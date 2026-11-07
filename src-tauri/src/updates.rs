// Updates — checks GitHub Releases (see plugins.updater in tauri.conf.json),
// then downloads, installs and restarts. Releases must be signed with the
// updater key, or the app refuses them.

use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

/// The update found by `check`, waiting for `install`.
#[derive(Default)]
pub struct Pending(Mutex<Option<Update>>);

/// Returns the new version, or None when already up to date.
pub async fn check(app: &AppHandle) -> Result<Option<String>, String> {
    let update = app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())?;
    let version = update.as_ref().map(|u| u.version.clone());
    *app.state::<Pending>().0.lock().unwrap() = update;
    Ok(version)
}

/// Emits "update-progress" (0–100) while downloading, then restarts into the new version.
pub async fn install(app: &AppHandle) -> Result<(), String> {
    let update = app.state::<Pending>().0.lock().unwrap().take().ok_or("no update to install")?;

    let mut received = 0usize;
    let progress = app.clone();
    update
        .download_and_install(
            move |chunk, total| {
                received += chunk;
                if let Some(total) = total.filter(|t| *t > 0) {
                    let _ = progress.emit("update-progress", (received as u64 * 100 / total).min(100));
                }
            },
            || {},
        )
        .await
        .map_err(|e| e.to_string())?;

    app.restart();
}
