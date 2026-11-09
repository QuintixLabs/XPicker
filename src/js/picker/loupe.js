// ============================================================
// loupe.js
// The loupe next to the cursor: the zoomed-in pixels and the color
// bar underneath with the value.
// ============================================================

// On Linux the screenshot is often drawn by the CPU, so redrawing it while the
// mouse moves makes the loupe lag behind (and a fast flick lands the cursor on
// it). There the loupe is its own small layer that just moves. Windows and
// macOS draw the screenshot fast, so there it's drawn straight onto it.

import { FORMATS, isLight } from '../shared/color-formats.js';
import { LINUX, ctx, markDirty, moveLayer, roundRect, shot, sizeLayer, snap } from './screen.js';

export const FONT = 'Outfit, system-ui, sans-serif';

const GRID = 13;    // zoomed pixels per side, odd so the cursor pixel sits in the middle
const CELL = 12;    // size of one zoomed pixel
const BAR = 30;     // color bar under the grid
const BORDER = 2;   // white frame
const RADIUS = 12;
const OFFSET = 28;  // distance from the cursor
const EDGE = 8;     // closest it gets to the screen edge

const SIZE = GRID * CELL;
const WIDTH = SIZE + BORDER * 2;
const HEIGHT = SIZE + BAR + BORDER * 2;
// Room around it for the shadow: 4px left and right, 1px above, 13px below.
const PAD = { left: 4, top: 1, right: 4, bottom: 13 };

const LAYERED = LINUX;
const layer = document.getElementById('loupe');
let layerCtx = null;

/** Called once the screenshot is loaded. */
export function prepare() {
  if (!LAYERED) return;
  layerCtx = sizeLayer(layer, PAD.left + WIDTH + PAD.right, PAD.top + HEIGHT + PAD.bottom);
  layer.hidden = false;
}

/** Bottom-right of the cursor; flips left or up when it would leave the screen. */
export function place(cursor) {
  const { cssWidth: w, cssHeight: h } = shot;
  let left = cursor.x + OFFSET;
  if (left + WIDTH > w - EDGE) left = cursor.x - OFFSET - WIDTH;
  let top = cursor.y + OFFSET;
  if (top + HEIGHT > h - EDGE) top = cursor.y - OFFSET - HEIGHT;
  left = Math.min(Math.max(left, EDGE), w - WIDTH - EDGE);
  top = Math.min(Math.max(top, EDGE), h - HEIGHT - EDGE);
  const rect = { x: left - PAD.left, y: top - PAD.top, w: PAD.left + WIDTH + PAD.right, h: PAD.top + HEIGHT + PAD.bottom };
  if (LAYERED) {
    // The layer sits on whole screen pixels, so it stays sharp at any scaling.
    rect.x = snap(rect.x);
    rect.y = snap(rect.y);
  }
  return { left, top, rect };
}

/** Draws the loupe at `left, top` around screenshot pixel `px, py`. */
export function draw({ left, top, rect }, px, py, color, formatIndex) {
  let g = ctx;
  if (LAYERED) {
    // Draw into the small layer (its own top left at 0, 0), then just move it.
    g = layerCtx;
    g.save();
    g.setTransform(1, 0, 0, 1, 0, 0);
    g.clearRect(0, 0, layer.width, layer.height);
    g.restore();
    left = PAD.left;
    top = PAD.top;
  }
  drawAt(g, left, top, px, py, color, formatIndex);
  if (LAYERED) moveLayer(layer, rect.x, rect.y);
  else markDirty(rect.x, rect.y, rect.w, rect.h);
}

function drawAt(g, left, top, px, py, color, formatIndex) {
  const gx = left + BORDER;
  const gy = top + BORDER;

  // White frame
  g.globalAlpha = 1;
  g.fillStyle = '#fff';
  roundRect(left, top, WIDTH, HEIGHT, RADIUS, g);
  g.fill();

  g.save();
  roundRect(gx, gy, SIZE, SIZE + BAR, RADIUS - BORDER, g);
  g.clip();

  // The zoomed pixels: one sharp (not blurred) copy of the area around the cursor.
  g.fillStyle = '#000';
  g.fillRect(gx, gy, SIZE, SIZE);
  const sx = px - (GRID >> 1);
  const sy = py - (GRID >> 1);
  const x0 = Math.max(0, sx);
  const y0 = Math.max(0, sy);
  const x1 = Math.min(shot.width, sx + GRID);
  const y1 = Math.min(shot.height, sy + GRID);
  g.imageSmoothingEnabled = false;
  g.drawImage(shot.bitmap, x0, y0, x1 - x0, y1 - y0, gx + (x0 - sx) * CELL, gy + (y0 - sy) * CELL, (x1 - x0) * CELL, (y1 - y0) * CELL);

  // Faint pixel grid.
  g.beginPath();
  for (let i = 1; i < GRID; i++) {
    g.moveTo(gx + i * CELL, gy);
    g.lineTo(gx + i * CELL, gy + SIZE);
    g.moveTo(gx, gy + i * CELL);
    g.lineTo(gx + SIZE, gy + i * CELL);
  }
  g.strokeStyle = 'rgba(128, 128, 128, 0.14)';
  g.lineWidth = 1;
  g.stroke();

  // Black and white frame around the picked pixel, so it shows on any color.
  const at = (GRID >> 1) * CELL;
  g.strokeStyle = '#000';
  g.strokeRect(gx + at - 0.5, gy + at - 0.5, CELL + 1, CELL + 1);
  g.strokeStyle = '#fff';
  g.strokeRect(gx + at + 0.5, gy + at + 0.5, CELL - 1, CELL - 1);

  // Color bar with the value.
  g.fillStyle = `#${color.hex}`;
  g.fillRect(gx, gy + SIZE, SIZE, BAR);
  g.fillStyle = isLight(color) ? '#000' : '#fff';
  g.font = `600 13px ${FONT}`;
  g.textAlign = 'center';
  g.textBaseline = 'middle';
  g.fillText(FORMATS[formatIndex].format(color), gx + SIZE / 2, gy + SIZE + BAR / 2 + 1, SIZE - 12);
  g.restore();

  g.strokeStyle = 'rgba(0, 0, 0, 0.45)';
  roundRect(left - 0.5, top - 0.5, WIDTH + 1, HEIGHT + 1, RADIUS + 0.5, g);
  g.stroke();
}
