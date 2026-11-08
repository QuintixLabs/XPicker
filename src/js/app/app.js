// ============================================================
// app.js
// Starts the main window: loads your settings and colors, then
// opens the right page.
// ============================================================

import '../shared/native.js';
import { app, listen, load } from './state.js';

// Each of these sets itself up and redraws when settings change.
// The theme goes first, so the page is in the right colors from the start.
import './components/theme.js';
import './components/dialog.js';
import './components/key-recorder.js';
import './components/update-bar.js';
import { show } from './components/sidebar.js';
import { goStep } from './pages/welcome.js';
import './pages/colors.js';
import './pages/settings.js';

await load();

// The tray's "Settings" opens the window straight on that page.
show(app.state.settings.showWelcome ? 'welcome' : (window.__START_PAGE__ ?? 'colors'));
listen('navigate', (e) => !app.state.settings.showWelcome && show(e.payload));
goStep(0);
