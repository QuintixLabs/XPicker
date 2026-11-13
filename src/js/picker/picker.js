// ============================================================
// picker/picker.js
// Starts the picker: loads the screenshot, sets up the loupe, the
// hint bar and your keys, then shows the window.
// ============================================================

import '../core/native.js';
import * as screen from './screen.js';
import * as loupe from './loupe.js';
import * as hints from './hints.js';
import * as crosshair from './crosshair.js';
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

  hints.update(spot.rect, cursor);
  loupe.draw(spot, px, py, screen.pixelAt(px, py), controls.currentFormat());
  crosshair.draw(px, py); // Linux only: moves in the same frame as the loupe
}

screen.setPainter(paint);

// ------------------------------------------------------------
// Starting
// ------------------------------------------------------------

async function begin() {
  const [raw, state] = await Promise.all([invoke('frame'), invoke('state'), document.fonts.load(`600 13px ${loupe.FONT}`)]);
  const buffer = raw instanceof ArrayBuffer ? raw : Uint8Array.from(raw).buffer;
  if (buffer.byteLength <= screen.HEADER) return; // no screenshot waiting

  const cursor = await screen.load(buffer);
  // Linux: the window's real size may already differ from what was reported.
  const before = screen.shot.scale;
  if (screen.LINUX && screen.fit()) {
    cursor.x = (cursor.x * before) / screen.shot.scale;
    cursor.y = (cursor.y * before) / screen.shot.scale;
  }
  controls.mouse.x = cursor.x;
  controls.mouse.y = cursor.y;
  controls.prepare(state.settings);
  hints.prepare(state.settings);
  loupe.prepare();
  crosshair.prepare();

  // Linux: the window only covers the screen once the desktop makes it
  // fullscreen, and then its real size can differ from what was reported.
  if (screen.LINUX) {
    addEventListener('resize', () => {
      if (!screen.fit()) return;
      hints.prepare(state.settings);
      loupe.prepare();
      crosshair.prepare();
      screen.request();
    });
  }

  screen.request();
  await invoke('ready');
}

begin();
