// ============================================================
// native.js
// Makes the windows feel like an app, not a web page: no browser
// shortcuts, right-click menu, zooming or dragging.
// ============================================================

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

// Right-click menu, side mouse buttons going "back", dragging and Ctrl+wheel zoom.
for (const type of ['contextmenu', 'auxclick', 'dragstart']) {
  addEventListener(type, (e) => e.preventDefault(), true);
}
addEventListener('wheel', (e) => e.ctrlKey && e.preventDefault(), { passive: false });

// Clicked buttons don't keep focus, so no focus ring is left behind on them.
addEventListener('mouseup', (e) => e.target.closest?.('button')?.blur());
