// ============================================================
// window/theme.js
// Light, dark, or the same as your system. "System" also follows
// the computer when it switches.
// ============================================================

// Windows and macOS tell the page when the system switches. On Linux the page
// can't always see it, so the app tells it instead (window.__SYSTEM_LIGHT__ at
// the start, then the "system-theme" event).

import { app, listen, onRender } from '../core/state.js';

const media = matchMedia('(prefers-color-scheme: light)');
let systemLight = window.__SYSTEM_LIGHT__ ?? media.matches;
let shown = null;

function apply(theme, fade) {
  const root = document.documentElement;
  const light = theme === 'light' || (theme === 'system' && systemLight);
  if (fade) {
    root.classList.add('theme-changing');
    setTimeout(() => root.classList.remove('theme-changing'), 250);
  }
  root.dataset.theme = light ? 'light' : 'dark';
  try {
    localStorage.setItem('xpicker.theme', theme); // lets boot.js open in the right theme
  } catch {}
}

/** The system switched: follow it if the theme is "System". */
function systemChanged(light) {
  if (light === systemLight) return;
  systemLight = light;
  if (app.state) apply(app.state.settings.theme, true);
}

// Fade only when the theme changes, not when the window opens.
onRender((state) => {
  if (state.settings.theme === shown) return;
  apply(state.settings.theme, shown !== null);
  shown = state.settings.theme;
});

if (window.__SYSTEM_LIGHT__ === undefined) {
  media.addEventListener('change', () => systemChanged(media.matches));
} else {
  listen('system-theme', (e) => systemChanged(e.payload));
}
