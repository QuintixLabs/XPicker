# Setup

XPicker is a [Tauri](https://tauri.app) app: the window, picker and notifications are plain HTML, CSS and JavaScript in `src/`, and a small Rust part in `src-tauri/` handles the screenshot, clipboard, shortcut and windows.

## What you need

- [Node.js](https://nodejs.org) 20 or newer
- [Rust](https://rustup.rs) (stable, 1.90 or newer)
- The [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your system. On Windows that's WebView2, which Windows 10 and 11 already have, plus the C++ Build Tools.

## Get the code

Fork XPicker on GitHub, then clone your fork and make a branch for your change:

```sh
git clone https://github.com/YOUR_NAME/XPicker.git
cd XPicker
git switch -c my-change
npm install
```

## Run it

```sh
npm run dev
```

The first run compiles the Rust part and takes a few minutes. After that it starts in seconds.

XPicker keeps running in the tray after you close its window. Quit it from the tray icon before running `npm run dev` again, or the new copy hands over to the old one.

## Where things are

| Path | What |
|---|---|
| `src/index.html`, `app.js` | Main window: welcome, colors, settings |
| `src/overlay.js` | The picker (frozen screenshot + loupe) |
| `src/styles/theme.css` | Colors, font and shared styles |
| `src-tauri/src/` | Rust: capture, clipboard, shortcut, windows, notifications, updates |

Notifications are real system notifications (`src-tauri/src/notify.rs`). In a dev build on Windows they show under PowerShell's name, since Windows only knows XPicker's own name once it's installed.

## Make a build

```sh
npm run build
```

The installer ends up in `src-tauri/target/release/bundle/`. Signing the update files needs the updater key, which only maintainers have, so a local build prints a signing error at the very end. The installer itself is still made and works.

## Your settings while testing

Settings and recent colors are saved in:

- **Windows:** `%APPDATA%\com.quintixlabs.xpicker\xpicker.json`
- **macOS:** `~/Library/Application Support/com.quintixlabs.xpicker/xpicker.json`
- **Linux:** `~/.config/com.quintixlabs.xpicker/xpicker.json`

Delete the file to start fresh with the welcome screen, or use **Settings → Show welcome screen** to see it again without losing anything.

## If something goes wrong

Errors from the Rust part show in the terminal running `npm run dev`. For the windows, right-click is turned off, so open the developer tools with `Ctrl+Shift+I` in a dev build. Include the useful parts when you open an issue.

---

#### [02. Committing ➔](/docs/contents/02_COMMITING.md)
