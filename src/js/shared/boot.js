// ============================================================
// boot.js
// Sets the theme and the sidebar before the window first appears,
// so it never flashes the wrong one. The welcome screen has no sidebar.
// ============================================================

// Loaded as a plain script in <head>, before anything is drawn.
// components/theme.js and components/sidebar.js keep these saved values up to date.
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
