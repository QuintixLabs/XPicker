// ============================================================
// picker.js
// Starts the picker: loads the screenshot, sets up the loupe, the
// hint bar and your keys, then shows the window.
// ============================================================

import '../shared/native.js';
import * as screen from './screen.js';
import * as loupe from './loupe.js';
import * as hints from './hints.js';
import * as controls from './controls.js';

const { invoke } = window.__TAURI__.core;

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
// Starting
// ------------------------------------------------------------

async function begin() {
  const [raw, state] = await Promise.all([invoke('frame'), invoke('state'), document.fonts.load(`600 13px ${loupe.FONT}`)]);
  const buffer = raw instanceof ArrayBuffer ? raw : Uint8Array.from(raw).buffer;
  if (buffer.byteLength <= 20) return; // no screenshot waiting

  const cursor = await screen.load(buffer);
  controls.mouse.x = cursor.x;
  controls.mouse.y = cursor.y;
  controls.prepare(state.settings);
  hints.prepare(state.settings);

  screen.request();
  await invoke('ready');
}

begin();
