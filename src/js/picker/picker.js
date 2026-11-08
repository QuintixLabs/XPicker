// ============================================================
// picker.js
// Starts the picker: loads the screenshot, sets up the loupe, the
// hint bar and your keys, then shows the window.
// ============================================================

// With "Instant picker" on, this page stays loaded in a hidden window and
// starts a new pick each time the app says "picker-open".

import '../shared/native.js';
import * as screen from './screen.js';
import * as loupe from './loupe.js';
import * as hints from './hints.js';
import * as controls from './controls.js';

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ------------------------------------------------------------
// Each frame
// ------------------------------------------------------------

function paint() {
  const { x: px, y: py } = screen.pixelUnder(controls.mouse);
  // Center of the picked pixel, in css px.
  const cursor = { x: (px + 0.5) / screen.shot.scale, y: (py + 0.5) / screen.shot.scale };
  const spot = loupe.place(cursor);

  const fading = hints.draw(spot.rect, cursor);
  loupe.draw(spot, px, py, screen.pixelAt(px, py), controls.currentFormat());

  // Keep drawing until the hint bar has finished fading, even if the mouse stops.
  if (fading) requestAnimationFrame(screen.request);
}

screen.setPainter(paint);

// ------------------------------------------------------------
// Starting and ending a pick
// ------------------------------------------------------------

let busy = false;

async function begin() {
  if (busy) return;
  busy = true;
  try {
    const [raw, state] = await Promise.all([invoke('frame'), invoke('state'), document.fonts.load(`600 13px ${loupe.FONT}`)]);
    const buffer = raw instanceof ArrayBuffer ? raw : Uint8Array.from(raw).buffer;
    if (buffer.byteLength <= 20) return; // a ready window, nothing to pick yet

    const cursor = await screen.load(buffer);
    controls.mouse.x = cursor.x;
    controls.mouse.y = cursor.y;
    controls.prepare(state.settings, end);
    hints.prepare(state.settings);

    screen.request();
    await invoke('ready');
  } finally {
    busy = false;
  }
}

/** The picker is hidden or closed: free the screenshot until next time. */
function end() {
  screen.unload();
}

listen('picker-open', begin);
begin();
