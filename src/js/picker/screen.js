// ============================================================
// picker/screen.js
// The screenshot under the picker: shows it, reads its pixels, and
// the small layers drawn on top of it (hint bar, Linux loupe).
// ============================================================

import { rgb } from '../core/color-formats.js';

const MIN_FRAME_MS = 4; // draw cap (~250 fps), plenty for high refresh monitors

/**
 * Linux hears about mouse moves a frame or so after the cursor moved, so
 * anything drawn next to the cursor trails it. There the loupe is its own
 * layer and XPicker draws its own crosshair (see loupe.js, crosshair.js).
 */
export const LINUX = /Linux/.test(navigator.userAgent);

export const canvas = document.getElementById('screen');
// "desynchronized" is the low-latency drawing mode, so a loupe drawn on the
// screenshot (Windows, macOS) keeps up with the mouse.
export const ctx = canvas.getContext('2d', { alpha: false, desynchronized: true });

/**
 * The current screenshot. Sizes in screenshot pixels, plus the window size in
 * css px and `room`: how much of each edge taskbars take, in css px.
 */
export const shot = { loaded: false, data: null, width: 0, height: 0, scale: 1, cssWidth: 0, cssHeight: 0, room: null, bitmap: null };

/** Size of the numbers in front of the pixels (see picker/overlay/capture.rs). */
export const HEADER = 36;

let dirty = []; // screenshot-pixel areas drawn over last frame

// ------------------------------------------------------------
// Loading
// ------------------------------------------------------------

/**
 * Loads a screenshot from the app: nine numbers [width, height, cursor x,
 * cursor y, scale × 1000, taskbar room left, top, right, bottom], then the
 * pixels. Returns the cursor in css px.
 */
export async function load(buffer) {
  const [width, height, x, y, scale1000, left, top, right, bottom] = new Int32Array(buffer, 0, 9);
  const data = new Uint8ClampedArray(buffer, HEADER);
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
    room: { left: left / scale, top: top / scale, right: right / scale, bottom: bottom / scale },
    bitmap: await createImageBitmap(image),
  });
  dirty = [];
  return { x: x / scale, y: y / scale };
}

// ------------------------------------------------------------
// Pixels
// ------------------------------------------------------------

export function pixelAt(x, y) {
  if (x < 0 || y < 0 || x >= shot.width || y >= shot.height) return rgb(0, 0, 0);
  const i = (y * shot.width + x) * 4;
  return rgb(shot.data[i], shot.data[i + 1], shot.data[i + 2]);
}

/**
 * The screenshot pixel under a point in css px. The tiny extra keeps a point
 * sitting exactly on a pixel's edge (like 2.9999… for 3) on that pixel.
 */
export function pixelUnder(point) {
  return {
    x: Math.min(Math.floor(point.x * shot.scale + 1e-6), shot.width - 1),
    y: Math.min(Math.floor(point.y * shot.scale + 1e-6), shot.height - 1),
  };
}

/**
 * The window as it really is. The scale comes from the system's idea of the
 * screen, which some Linux desktops get wrong (with scaling); once the window
 * covers the screen, its own width is the truth. True if anything changed, so
 * the layers can be set up again.
 */
export function fit() {
  const width = canvas.clientWidth;
  if (!shot.loaded || !width || Math.abs(width - shot.cssWidth) < 0.5) return false;
  const scale = shot.width / width;
  const room = shot.room;
  Object.assign(shot, {
    scale,
    cssWidth: width,
    cssHeight: shot.height / scale,
    room: { left: (room.left * shot.scale) / scale, top: (room.top * shot.scale) / scale, right: (room.right * shot.scale) / scale, bottom: (room.bottom * shot.scale) / scale },
  });
  return true;
}

// ------------------------------------------------------------
// Layers: small canvases on top of the screenshot
// ------------------------------------------------------------

/**
 * Gives a layer room for `w` × `h` css px, sharp on this screen. Returns its
 * context, ready to draw in css px with (0, 0) at the layer's top left.
 */
export function sizeLayer(layer, w, h) {
  const s = shot.scale;
  layer.width = Math.ceil(w * s);
  layer.height = Math.ceil(h * s);
  layer.style.width = `${layer.width / s}px`;
  layer.style.height = `${layer.height / s}px`;
  const g = layer.getContext('2d');
  g.setTransform(s, 0, 0, s, 0, 0);
  return g;
}

/** Rounds a css position to whole screen pixels, so layers stay sharp. */
export const snap = (v) => Math.round(v * shot.scale) / shot.scale;

/** Moves a layer to `x, y` (css px). Only moves it on screen, nothing gets redrawn. */
export function moveLayer(layer, x, y) {
  layer.style.transform = `translate(${x}px, ${y}px)`;
}

// ------------------------------------------------------------
// Drawing on the screenshot: only what changed
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

/** A rounded rectangle path on `g` (the screenshot's context unless given). */
export function roundRect(x, y, w, h, r, g = ctx) {
  g.beginPath();
  g.roundRect(x, y, w, h, r);
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
