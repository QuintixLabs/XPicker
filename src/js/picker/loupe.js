// ============================================================
// loupe.js
// The loupe next to the cursor: the zoomed-in pixels and the color
// bar underneath with the value.
// ============================================================

import { FORMATS, isLight } from '../shared/color-formats.js';
import { ctx, markDirty, roundRect, shot } from './screen.js';

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

/** Bottom-right of the cursor; flips left or up when it would leave the screen. */
export function place(cursor) {
  const { cssWidth: w, cssHeight: h } = shot;
  let left = cursor.x + OFFSET;
  if (left + WIDTH > w - EDGE) left = cursor.x - OFFSET - WIDTH;
  let top = cursor.y + OFFSET;
  if (top + HEIGHT > h - EDGE) top = cursor.y - OFFSET - HEIGHT;
  left = Math.min(Math.max(left, EDGE), w - WIDTH - EDGE);
  top = Math.min(Math.max(top, EDGE), h - HEIGHT - EDGE);
  return { left, top, rect: { x: left - 4, y: top - 1, w: WIDTH + 8, h: HEIGHT + 13 } };
}

/** Draws the loupe at `left, top` around screenshot pixel `px, py`. */
export function draw({ left, top }, px, py, color, formatIndex) {
  const gx = left + BORDER;
  const gy = top + BORDER;

  // Soft shadow, then the white frame.
  ctx.globalAlpha = 1;
  ctx.fillStyle = 'rgba(0, 0, 0, 0.14)';
  roundRect(left - 3, top + 3, WIDTH + 6, HEIGHT + 8, RADIUS + 3);
  ctx.fill();
  ctx.fillStyle = 'rgba(0, 0, 0, 0.22)';
  roundRect(left - 1, top + 1, WIDTH + 2, HEIGHT + 3, RADIUS + 1);
  ctx.fill();
  ctx.fillStyle = '#fff';
  roundRect(left, top, WIDTH, HEIGHT, RADIUS);
  ctx.fill();

  ctx.save();
  roundRect(gx, gy, SIZE, SIZE + BAR, RADIUS - BORDER);
  ctx.clip();

  // The zoomed pixels: one sharp (not blurred) copy of the area around the cursor.
  ctx.fillStyle = '#000';
  ctx.fillRect(gx, gy, SIZE, SIZE);
  const sx = px - (GRID >> 1);
  const sy = py - (GRID >> 1);
  const x0 = Math.max(0, sx);
  const y0 = Math.max(0, sy);
  const x1 = Math.min(shot.width, sx + GRID);
  const y1 = Math.min(shot.height, sy + GRID);
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(shot.bitmap, x0, y0, x1 - x0, y1 - y0, gx + (x0 - sx) * CELL, gy + (y0 - sy) * CELL, (x1 - x0) * CELL, (y1 - y0) * CELL);

  // Faint pixel grid.
  ctx.beginPath();
  for (let i = 1; i < GRID; i++) {
    ctx.moveTo(gx + i * CELL, gy);
    ctx.lineTo(gx + i * CELL, gy + SIZE);
    ctx.moveTo(gx, gy + i * CELL);
    ctx.lineTo(gx + SIZE, gy + i * CELL);
  }
  ctx.strokeStyle = 'rgba(128, 128, 128, 0.14)';
  ctx.lineWidth = 1;
  ctx.stroke();

  // Black and white frame around the picked pixel, so it shows on any color.
  const at = (GRID >> 1) * CELL;
  ctx.strokeStyle = '#000';
  ctx.strokeRect(gx + at - 0.5, gy + at - 0.5, CELL + 1, CELL + 1);
  ctx.strokeStyle = '#fff';
  ctx.strokeRect(gx + at + 0.5, gy + at + 0.5, CELL - 1, CELL - 1);

  // Color bar with the value.
  ctx.fillStyle = `#${color.hex}`;
  ctx.fillRect(gx, gy + SIZE, SIZE, BAR);
  ctx.fillStyle = isLight(color) ? '#000' : '#fff';
  ctx.font = `600 13px ${FONT}`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(FORMATS[formatIndex].format(color), gx + SIZE / 2, gy + SIZE + BAR / 2 + 1, SIZE - 12);
  ctx.restore();

  ctx.strokeStyle = 'rgba(0, 0, 0, 0.45)';
  roundRect(left - 0.5, top - 0.5, WIDTH + 1, HEIGHT + 1, RADIUS + 0.5);
  ctx.stroke();

  markDirty(left - 4, top - 1, WIDTH + 8, HEIGHT + 13);
}
