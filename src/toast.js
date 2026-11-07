// Toast — preloaded (invisible) while picking; shows the instant a color is picked.
// Click to open XPicker.

import './lib/native.js';

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const LEAVE_MS = 180; // keep in sync with .is-leaving in toast.css

const toast = document.getElementById('toast');
let shown = false;
let closing = false;

function show({ value, hex, ms }) {
  if (shown) return;
  shown = true;

  const n = parseInt(hex, 16);
  const swatch = document.getElementById('swatch');
  swatch.style.setProperty('--color', `#${hex}`);
  swatch.classList.toggle('is-dark', (n >> 16) + ((n >> 8) & 255) + (n & 255) < 60);
  document.getElementById('value').textContent = value;

  requestAnimationFrame(() => toast.classList.add('is-visible'));
  setTimeout(close, ms);
}

function close() {
  if (closing) return;
  closing = true;
  toast.classList.remove('is-visible');
  toast.classList.add('is-leaving');
  setTimeout(() => invoke('dismiss'), LEAVE_MS);
}

toast.addEventListener('click', () => {
  if (!shown) return;
  invoke('open_main');
  close();
});

// The pick usually arrives as an event; if it happened before this page loaded, fetch it.
listen('toast', (e) => show(e.payload));
invoke('toast_payload').then((payload) => payload && show(payload));
