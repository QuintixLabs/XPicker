// ============================================================
// theme.js
// Light, dark, or the same as your system. "System" also follows
// the computer when it switches.
// ============================================================

import { app, onRender } from '../state.js';

const systemLight = matchMedia('(prefers-color-scheme: light)');
let shown = null;

function apply(theme, fade) {
  const root = document.documentElement;
  const light = theme === 'light' || (theme === 'system' && systemLight.matches);
  if (fade) {
    root.classList.add('theme-changing');
    setTimeout(() => root.classList.remove('theme-changing'), 250);
  }
  root.dataset.theme = light ? 'light' : 'dark';
  try {
    localStorage.setItem('xpicker.theme', theme); // lets boot.js open in the right theme
  } catch {}
}

// Fade only when the theme changes, not when the window opens.
onRender((state) => {
  if (state.settings.theme === shown) return;
  apply(state.settings.theme, shown !== null);
  shown = state.settings.theme;
});

systemLight.addEventListener('change', () => app.state && apply(app.state.settings.theme, true));
