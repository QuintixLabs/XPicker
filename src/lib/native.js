// Makes the webview behave like a desktop app: no browser shortcuts,
// right-click menu, zooming or dragging. Imported by every window.

const BLOCKED_KEYS = new Set(['F3', 'F5', 'F7', 'F12', 'BrowserBack', 'BrowserForward', 'BrowserRefresh']);
const BLOCKED_CTRL = new Set(['a', 'f', 'g', 'j', 'p', 'r', 's', 'u', '+', '-', '=', '0']);

addEventListener(
  'keydown',
  (e) => {
    const ctrl = e.ctrlKey || e.metaKey;
    if (BLOCKED_KEYS.has(e.key) || (ctrl && BLOCKED_CTRL.has(e.key.toLowerCase()))) e.preventDefault();
  },
  true,
);

addEventListener('contextmenu', (e) => e.preventDefault());
addEventListener('dragstart', (e) => e.preventDefault());
addEventListener('wheel', (e) => e.ctrlKey && e.preventDefault(), { passive: false });
