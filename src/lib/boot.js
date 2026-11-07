// Runs before the first paint (classic script in <head>) so the window opens
// already in its saved theme and sidebar state, with no flash of the other one.
// app.js keeps these two localStorage copies in sync with the real settings.
try {
  const root = document.documentElement;
  const theme = localStorage.getItem('xpicker.theme') ?? 'system';
  const light = theme === 'light' || (theme === 'system' && matchMedia('(prefers-color-scheme: light)').matches);
  root.dataset.theme = light ? 'light' : 'dark';

  if (localStorage.getItem('xpicker.sidebarCollapsed') === '1') {
    root.classList.add('is-collapsed');
  }
} catch {}
