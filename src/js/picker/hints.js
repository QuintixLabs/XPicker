// ============================================================
// picker/hints.js
// The hint bar: your keys at the bottom while picking.
// ============================================================

// It's its own small layer: drawn once when the picker opens, then only faded
// (by CSS, see picker.css), so it costs nothing while you move the mouse.

import { bindingKeys } from '../core/bindings.js';
import { FONT } from './loupe.js';
import { moveLayer, roundRect, shot, sizeLayer, snap } from './screen.js';

const HEIGHT = 46;
const MARGIN = 24;           // from the screen edge (or the taskbar)
const KEY_FONT = `600 12px ${FONT}`;
const LABEL_FONT = `500 14px ${FONT}`;
const NEAR = 12;             // this close and it fades out
const AWAY = 48;             // this far and it comes back, so it doesn't flicker at the edge

// Follows the app's theme ("system" = the computer's setting).
const COLORS = {
  dark: { bg: 'rgba(14, 14, 16, 0.9)', border: 'rgba(255, 255, 255, 0.08)', cap: 'rgba(255, 255, 255, 0.1)', key: '#f4f4f5', label: '#a1a1a8' },
  light: { bg: 'rgba(255, 255, 255, 0.94)', border: 'rgba(0, 0, 0, 0.1)', cap: 'rgba(0, 0, 0, 0.06)', key: '#18181b', label: '#6b6b74' },
};

const layer = document.getElementById('hints');
let width = 0;
let shown = true;

// ------------------------------------------------------------
// Setting up
// ------------------------------------------------------------

/** Measures, draws and places the bar for these keys. Called once the screenshot is loaded. */
export function prepare(settings) {
  // On Linux the app says whether the desktop is light (see window/mod.rs).
  const systemLight = window.__SYSTEM_LIGHT__ ?? matchMedia('(prefers-color-scheme: light)').matches;
  const light = settings.theme === 'light' || (settings.theme === 'system' && systemLight);
  const colors = light ? COLORS.light : COLORS.dark;

  const g = layer.getContext('2d');
  const items = [
    [bindingKeys(settings.pick), 'Copy'],
    [bindingKeys(settings.cycle), 'Format'],
    [bindingKeys(settings.cancel), 'Cancel'],
    [bindingKeys(settings.open), 'Open XPicker'],
    [['↑↓←→'], 'Move'], // arrow keys: 1 pixel, with Shift 10 (controls.js)
  ].map(([caps, label]) => {
    g.font = KEY_FONT;
    const capWidths = caps.map((c) => g.measureText(c).width + 16);
    g.font = LABEL_FONT;
    const labelWidth = g.measureText(label).width;
    const itemWidth = capWidths.reduce((a, b) => a + b + 4, -4) + 8 + labelWidth;
    return { caps, capWidths, label, labelWidth, width: itemWidth };
  });
  width = Math.ceil(items.reduce((a, item) => a + item.width + 24, -24) + 36);

  drawBar(sizeLayer(layer, width + 2, HEIGHT + 2), items, colors);
  const { x, y } = rect();
  moveLayer(layer, snap(x - 1), snap(y - 1));
  shown = true;
  layer.classList.remove('is-away');
}

/** Centered in the part of the screen taskbars leave free, just above the bottom one. */
function rect(grow = 0) {
  const { left, right, bottom } = shot.room;
  const x = left + (shot.cssWidth - left - right - width) / 2;
  const y = shot.cssHeight - bottom - MARGIN - HEIGHT;
  return { x: x - grow, y: y - grow, w: width + grow * 2, h: HEIGHT + grow * 2 };
}

function drawBar(g, items, colors) {
  // Drawn at (1, 1) in the layer, leaving room for the 1px border.
  g.fillStyle = colors.bg;
  roundRect(1, 1, width, HEIGHT, 12, g);
  g.fill();
  g.strokeStyle = colors.border;
  g.lineWidth = 1;
  g.stroke();

  g.textBaseline = 'middle';
  g.textAlign = 'left';
  let x = 19;
  const mid = 1 + HEIGHT / 2 + 1;
  for (const item of items) {
    item.caps.forEach((cap, i) => {
      const w = item.capWidths[i];
      g.fillStyle = colors.cap;
      roundRect(x, 10, w, HEIGHT - 18, 6, g);
      g.fill();
      g.fillStyle = colors.key;
      g.font = KEY_FONT;
      g.fillText(cap, x + 8, mid);
      x += w + 4;
    });
    x += 4;
    g.fillStyle = colors.label;
    g.font = LABEL_FONT;
    g.fillText(item.label, x, mid);
    x += item.labelWidth + 24;
  }
}

// ------------------------------------------------------------
// Fading out of the way
// ------------------------------------------------------------

const overlaps = (a, b) => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
const contains = (r, p) => p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h;

function nearby(loupe, cursor, grow) {
  const area = rect(grow);
  return overlaps(area, loupe) || contains(area, cursor);
}

/** Fades the bar out when the loupe or cursor comes near, and back once they've left. */
export function update(loupe, cursor) {
  if (shown && nearby(loupe, cursor, NEAR)) shown = false;
  else if (!shown && !nearby(loupe, cursor, AWAY)) shown = true;
  layer.classList.toggle('is-away', !shown);
}
