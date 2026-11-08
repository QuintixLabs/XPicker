// ============================================================
// screen.js
// The screenshot under the picker: holds it, reads its pixels, and
// redraws only the parts that changed each frame.
// ============================================================

import { rgb } from '../shared/color-formats.js';

const MIN_FRAME_MS = 4; // draw cap (~250 fps), plenty for high refresh monitors

export const canvas = document.getElementById('canvas');
// "desynchronized" is the low-latency drawing mode, so the loupe keeps up with the mouse.
export const ctx = canvas.getContext('2d', { alpha: false, desynchronized: true });

/** The current screenshot. Sizes in screenshot pixels, plus the window size in css px. */
export const shot = { loaded: false, data: null, width: 0, height: 0, scale: 1, cssWidth: 0, cssHeight: 0, bitmap: null };

let dirty = []; // screenshot-pixel areas drawn over last frame

// ------------------------------------------------------------
// Loading
// ------------------------------------------------------------

/**
 * Loads a screenshot from the app: five numbers [width, height, cursor x,
 * cursor y, scale × 1000], then the pixels. Returns the cursor in css px.
 */
export async function load(buffer) {
  const [width, height, x, y, scale1000] = new Int32Array(buffer, 0, 5);
  const data = new Uint8ClampedArray(buffer, 20);
  const image = new ImageData(data, width, height);

  canvas.width = width;
  canvas.height = height;
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.putImageData(image, 0, 0);

  const scale = scale1000 / 1000;
  Object.assign(shot, {
    loaded: true,
    data,
    width,
    height,
    scale,
    cssWidth: width / scale,
    cssHeight: height / scale,
    bitmap: await createImageBitmap(image),
  });
  dirty = [];
  return { x: x / scale, y: y / scale };
}

/** Frees the screenshot once the picker is hidden (a big screen is many megabytes). */
export function unload() {
  shot.bitmap?.close();
  Object.assign(shot, { loaded: false, data: null, bitmap: null });
  canvas.width = 1;
  canvas.height = 1;
  dirty = [];
}

// ------------------------------------------------------------
// Pixels
// ------------------------------------------------------------

export function pixelAt(x, y) {
  if (x < 0 || y < 0 || x >= shot.width || y >= shot.height) return rgb(0, 0, 0);
  const i = (y * shot.width + x) * 4;
  return rgb(shot.data[i], shot.data[i + 1], shot.data[i + 2]);
}

/** The screenshot pixel under a point in css px. */
export function pixelUnder(point) {
  return {
    x: Math.min(Math.floor(point.x * shot.scale), shot.width - 1),
    y: Math.min(Math.floor(point.y * shot.scale), shot.height - 1),
  };
}

// ------------------------------------------------------------
// Drawing only what changed
// ------------------------------------------------------------

/** Remembers an area (css px) drawn this frame, to put the screenshot back there next frame. */
export function markDirty(x, y, w, h) {
  const s = shot.scale;
  dirty.push({ x: Math.floor(x * s) - 2, y: Math.floor(y * s) - 2, w: Math.ceil(w * s) + 4, h: Math.ceil(h * s) + 4 });
}

function restore({ x, y, w, h }) {
  const x0 = Math.max(0, x);
  const y0 = Math.max(0, y);
  const x1 = Math.min(shot.width, x + w);
  const y1 = Math.min(shot.height, y + h);
  if (x1 > x0 && y1 > y0) ctx.drawImage(shot.bitmap, x0, y0, x1 - x0, y1 - y0, x0, y0, x1 - x0, y1 - y0);
}

export function roundRect(x, y, w, h, r) {
  ctx.beginPath();
  ctx.roundRect(x, y, w, h, r);
}

// ------------------------------------------------------------
// The frame loop
// ------------------------------------------------------------

let paint = () => {};
let lastDraw = 0;
let pending = false;

/** What to draw each frame (set by picker.js). */
export function setPainter(fn) {
  paint = fn;
}

function draw() {
  pending = false;
  if (!shot.loaded) return;
  lastDraw = performance.now();

  ctx.setTransform(1, 0, 0, 1, 0, 0);
  for (const rect of dirty) restore(rect);
  dirty = [];

  ctx.setTransform(shot.scale, 0, 0, shot.scale, 0, 0);
  paint();
}

/** Draws now if it can, otherwise right after the frame cap. */
export function request() {
  if (!shot.loaded || pending) return;
  const wait = MIN_FRAME_MS - (performance.now() - lastDraw);
  if (wait <= 0) {
    draw();
  } else {
    pending = true;
    setTimeout(draw, wait);
  }
}
