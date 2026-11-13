// ============================================================
// main.js
// Starts the main window: loads your settings and colors, then
// opens the right page.
// ============================================================

import './core/native.js';
import { app, invoke, listen, load } from './core/state.js';
import { confirmAction } from './window/dialog.js';

// Each of these sets itself up and redraws when settings change.
// The theme goes first, so the page is in the right colors from the start.
import './window/theme.js';
import './window/dialog.js';
import './controls/key-recorder.js';
import './window/update-bar.js';
import './window/setup.js';
import { show } from './window/sidebar.js';
import { goStep } from './pages/welcome.js';
import './pages/colors.js';
import './pages/settings.js';

await load();

// The tray's "Settings" opens the window straight on that page.
show(app.state.settings.showWelcome ? 'welcome' : (window.__START_PAGE__ ?? 'colors'));
listen('navigate', (e) => !app.state.settings.showWelcome && show(e.payload));
goStep(0);

// macOS: a pick was stopped because XPicker can't see the screen yet.
async function screenPermission() {
  if (!(await invoke('screen_permission'))) return;
  const ok = await confirmAction({
    title: 'Let XPicker see your screen',
    text: 'To pick colors, turn on XPicker in System Settings, under Privacy & Security → Screen Recording. Then quit XPicker and open it again.',
    action: 'Open System Settings',
    danger: false,
  });
  if (ok) invoke('open_screen_settings');
}
listen('screen-permission', screenPermission);
screenPermission();
