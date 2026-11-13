// ============================================================
// core/boot.js
// Sets the theme before the window appears, so it never flashes.
// ============================================================

// Loaded as a plain script in <head>, before anything is drawn.
// window/theme.js and window/sidebar.js keep these saved values up to date.
try {
  const root = document.documentElement;
  const theme = localStorage.getItem('xpicker.theme') ?? 'system';
  // On Linux the app says whether the desktop is light (window.__SYSTEM_LIGHT__).
  const systemLight = window.__SYSTEM_LIGHT__ ?? matchMedia('(prefers-color-scheme: light)').matches;
  const light = theme === 'light' || (theme === 'system' && systemLight);
  root.dataset.theme = light ? 'light' : 'dark';

  if (localStorage.getItem('xpicker.sidebarCollapsed') === '1') {
    root.classList.add('is-collapsed');
  }
  if (window.__START_PAGE__ === 'welcome') {
    root.classList.add('is-welcome');
  }
} catch {}
