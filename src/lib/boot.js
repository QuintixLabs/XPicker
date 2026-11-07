// Runs before the first paint (classic script in <head>) so the window opens
// with the sidebar already in its saved state, with no flash of the other one.
try {
  if (localStorage.getItem('xpicker.sidebarCollapsed') === '1') {
    document.documentElement.classList.add('is-collapsed');
  }
} catch {}
