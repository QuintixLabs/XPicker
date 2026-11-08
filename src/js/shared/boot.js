// ============================================================
// boot.js
// Sets the theme and the sidebar before the window first appears,
// so it never flashes the wrong one.
// ============================================================

// Loaded as a plain script in <head>, before anything is drawn.
// components/theme.js and components/sidebar.js keep these saved values up to date.
try {
  const root = document.documentElement;
  const theme = localStorage.getItem('xpicker.theme') ?? 'system';
  const light = theme === 'light' || (theme === 'system' && matchMedia('(prefers-color-scheme: light)').matches);
  root.dataset.theme = light ? 'light' : 'dark';

  if (localStorage.getItem('xpicker.sidebarCollapsed') === '1') {
    root.classList.add('is-collapsed');
  }
} catch {}
