// Overlay — frozen screenshot, loupe and key hints on ONE canvas.
//
// The loupe sits beside the system crosshair and flips sides near screen edges.
// The canvas is "desynchronized" (low-latency mode), and each frame only
// repaints the small areas that changed.

import './lib/native.js';
import { FORMATS, rgb, isLight } from './lib/color.js';
import { fromKey, fromMouse, bindingKeys } from './lib/keys.js';

const { invoke } = window.__TAURI__.core;

const GRID = 13;          // magnified pixels per side, odd so the cursor pixel is centered
const CELL = 12;          // px per magnified pixel
const BAR = 30;           // color bar under the grid
const BORDER = 2;         // white frame
const RADIUS = 12;
const OFFSET = 28;        // loupe distance from the cursor
const EDGE = 8;           // min distance from screen edges
const CYCLE_MS = 240;     // format cycle rate while the key is held
const MIN_FRAME_MS = 4;   // draw cap (~250 fps), plenty for high refresh monitors
const FONT = 'Outfit, system-ui, sans-serif';

const SIZE = GRID * CELL;
const LOUPE_W = SIZE + BORDER * 2;
const LOUPE_H = SIZE + BAR + BORDER * 2;

const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d', { alpha: false, desynchronized: true });

let shot = null;             // { data, width, height, scale, bitmap }
let keys = {};               // { pick, cycle, cancel }
let hints = null;            // pre-measured hint layout
const mouse = { x: 0, y: 0 }; // css px
let formatIndex = 0;
let lastCycle = 0;
let picked = false;
let dirty = [];              // physical-px rects drawn last frame
let lastDraw = 0;
let pending = false;
let hintsOnTop = false;

// The hint bar follows the app's theme ("system" = the computer's setting).
// The loupe stays the same in both: it has to read on any screen content.
const HINT_COLORS = {
  dark: { bg: 'rgba(14, 14, 16, 0.9)', border: 'rgba(255, 255, 255, 0.08)', cap: 'rgba(255, 255, 255, 0.1)', key: '#f4f4f5', label: '#a1a1a8' },
  light: { bg: 'rgba(255, 255, 255, 0.94)', border: 'rgba(0, 0, 0, 0.1)', cap: 'rgba(0, 0, 0, 0.06)', key: '#18181b', label: '#6b6b74' },
};
let hintColors = HINT_COLORS.dark;
let armed = null;            // mouse binding pressed down, fires on release

// ---------- pixels

function pixelAt(x, y) {
  if (x < 0 || y < 0 || x >= shot.width || y >= shot.height) return rgb(0, 0, 0);
  const i = (y * shot.width + x) * 4;
  return rgb(shot.data[i], shot.data[i + 1], shot.data[i + 2]);
}

/** Screenshot pixel under the cursor. */
const target = () => ({
  x: Math.min(Math.floor(mouse.x * shot.scale), shot.width - 1),
  y: Math.min(Math.floor(mouse.y * shot.scale), shot.height - 1),
});

// ---------- dirty-rect bookkeeping

function markDirty(x, y, w, h) {
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

// ---------- drawing

function roundRect(x, y, w, h, r) {
  ctx.beginPath();
  ctx.roundRect(x, y, w, h, r);
}

/** Bottom-right of the cursor; flips left/up when it would leave the screen. */
function loupePosition(cx, cy) {
  let left = cx + OFFSET;
  if (left + LOUPE_W > innerWidth - EDGE) left = cx - OFFSET - LOUPE_W;
  let top = cy + OFFSET;
  if (top + LOUPE_H > innerHeight - EDGE) top = cy - OFFSET - LOUPE_H;
  return {
    left: Math.min(Math.max(left, EDGE), innerWidth - LOUPE_W - EDGE),
    top: Math.min(Math.max(top, EDGE), innerHeight - LOUPE_H - EDGE),
  };
}

function drawLoupe(left, top, px, py, color) {
  const gx = left + BORDER;
  const gy = top + BORDER;

  // Soft shadow, then the white frame.
  ctx.fillStyle = 'rgba(0, 0, 0, 0.14)';
  roundRect(left - 3, top + 3, LOUPE_W + 6, LOUPE_H + 8, RADIUS + 3);
  ctx.fill();
  ctx.fillStyle = 'rgba(0, 0, 0, 0.22)';
  roundRect(left - 1, top + 1, LOUPE_W + 2, LOUPE_H + 3, RADIUS + 1);
  ctx.fill();
  ctx.fillStyle = '#fff';
  roundRect(left, top, LOUPE_W, LOUPE_H, RADIUS);
  ctx.fill();

  ctx.save();
  roundRect(gx, gy, SIZE, SIZE + BAR, RADIUS - BORDER);
  ctx.clip();

  // Magnified pixels: one nearest-neighbour blit of the visible part.
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

  // Two-tone frame around the picked pixel, readable on any color.
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
  roundRect(left - 0.5, top - 0.5, LOUPE_W + 1, LOUPE_H + 1, RADIUS + 0.5);
  ctx.stroke();

  markDirty(left - 4, top - 1, LOUPE_W + 8, LOUPE_H + 13);
}

const overlaps = (a, b) => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;

// Hint bar sizes (css px).
const HINT_H = 46;
const HINT_MARGIN = 24;       // from the screen edge
const HINT_KEY_FONT = `600 12px ${FONT}`;
const HINT_LABEL_FONT = `500 14px ${FONT}`;
const HINT_SLIDE = 0.22;      // share of the remaining distance moved per frame

let hintY = null;             // where the bar is drawn now, gliding toward its target

/** The hint bar sits at the bottom unless the loupe actually reaches it; then it goes to the top. */
function hintsTarget(loupe) {
  const { width, height } = hints;
  const margin = hintsOnTop ? 32 : 12; // a little stickiness so it doesn't flicker
  const bottom = { x: (innerWidth - width) / 2 - margin, y: innerHeight - HINT_MARGIN - height - margin, w: width + margin * 2, h: height + margin * 2 };
  hintsOnTop = overlaps(bottom, loupe);
  return hintsOnTop ? HINT_MARGIN : innerHeight - HINT_MARGIN - height;
}

/** Moves the bar a step toward its target. Returns true while it's still moving. */
function slideHints(target) {
  if (hintY === null || Math.abs(target - hintY) < 0.5) {
    hintY = target;
    return false;
  }
  hintY += (target - hintY) * HINT_SLIDE;
  return true;
}

function measureHints() {
  const items = [
    [bindingKeys(keys.pick), 'Copy'],
    [bindingKeys(keys.cycle), 'Format'],
    [bindingKeys(keys.cancel), 'Cancel'],
    [bindingKeys(keys.open), 'Open XPicker'],
  ].map(([caps, label]) => {
    ctx.font = HINT_KEY_FONT;
    const capWidths = caps.map((c) => ctx.measureText(c).width + 16);
    ctx.font = HINT_LABEL_FONT;
    const labelWidth = ctx.measureText(label).width;
    const width = capWidths.reduce((a, b) => a + b + 4, -4) + 8 + labelWidth;
    return { caps, capWidths, label, width };
  });
  const width = items.reduce((a, item) => a + item.width + 24, -24) + 36;
  return { items, width, height: HINT_H };
}

function drawHints(y) {
  const { items, width, height } = hints;
  const x = Math.round((innerWidth - width) / 2);
  y = Math.round(y);

  ctx.fillStyle = hintColors.bg;
  roundRect(x, y, width, height, 12);
  ctx.fill();
  ctx.strokeStyle = hintColors.border;
  ctx.stroke();

  ctx.textBaseline = 'middle';
  ctx.textAlign = 'left';
  let cursor = x + 18;
  const mid = y + height / 2 + 1;
  for (const item of items) {
    item.caps.forEach((cap, i) => {
      const w = item.capWidths[i];
      ctx.fillStyle = hintColors.cap;
      roundRect(cursor, y + 9, w, height - 18, 6);
      ctx.fill();
      ctx.fillStyle = hintColors.key;
      ctx.font = HINT_KEY_FONT;
      ctx.fillText(cap, cursor + 8, mid);
      cursor += w + 4;
    });
    cursor += 4;
    ctx.fillStyle = hintColors.label;
    ctx.font = HINT_LABEL_FONT;
    ctx.fillText(item.label, cursor, mid);
    cursor += ctx.measureText(item.label).width + 24;
  }

  markDirty(x - 1, y - 1, width + 2, height + 2);
}

function draw() {
  pending = false;
  lastDraw = performance.now();
  const s = shot.scale;

  // Put back the screenshot where we drew last time.
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  for (const rect of dirty) restore(rect);
  dirty = [];

  ctx.setTransform(s, 0, 0, s, 0, 0);
  const { x: px, y: py } = target();
  const cx = (px + 0.5) / s; // center of the picked pixel, in css px
  const cy = (py + 0.5) / s;
  const { left, top } = loupePosition(cx, cy);
  const loupe = { x: left - 4, y: top - 1, w: LOUPE_W + 8, h: LOUPE_H + 13 };

  const sliding = slideHints(hintsTarget(loupe));
  drawHints(hintY);
  drawLoupe(left, top, px, py, pixelAt(px, py));

  // Keep drawing until the bar has finished gliding, even if the mouse stops.
  if (sliding) requestAnimationFrame(request);
}

/** Draw now if we can, otherwise right after the frame cap. */
function request() {
  if (!shot || pending) return;
  const wait = MIN_FRAME_MS - (performance.now() - lastDraw);
  if (wait <= 0) {
    draw();
  } else {
    pending = true;
    setTimeout(draw, wait);
  }
}

// ---------- actions

function pick() {
  if (picked) return;
  picked = true;
  const { x, y } = target();
  const color = pixelAt(x, y);
  const format = FORMATS[formatIndex];
  invoke('pick', { value: format.format(color), hex: color.hex, format: formatIndex });
}

function cycle(repeat) {
  const now = performance.now();
  if (repeat && now - lastCycle < CYCLE_MS) return;
  lastCycle = now;
  formatIndex = (formatIndex + 1) % FORMATS.length;
  request();
}

function actionFor(binding, e) {
  return {
    [keys.pick]: pick,
    [keys.cycle]: () => cycle(e.repeat),
    [keys.cancel]: () => invoke('dismiss'),
    [keys.open]: () => invoke('open_from_picker'),
  }[binding];
}

function handle(binding, e) {
  if (!shot) return;
  const action = actionFor(binding, e);
  if (!action) return;
  e.preventDefault();
  action();
}

// Windows sometimes refuses the first focus request; keep asking so keys
// (Esc especially) always reach the picker.
let lastFocusTry = 0;
function ensureFocus() {
  const now = performance.now();
  if (document.hasFocus() || now - lastFocusTry < 400) return;
  lastFocusTry = now;
  invoke('ready');
}

addEventListener('keydown', (e) => handle(fromKey(e), e));

// Mouse bindings fire on release: if we closed on press, the release would land
// on the app underneath (a right-click there opens its context menu).
addEventListener('mousedown', (e) => {
  e.preventDefault();
  const binding = fromMouse(e);
  if (shot && actionFor(binding, e)) armed = binding;
});
addEventListener('mouseup', (e) => {
  e.preventDefault();
  const binding = fromMouse(e);
  if (binding !== armed) return;
  armed = null;
  handle(binding, e);
});

// pointerrawupdate fires at the mouse's own polling rate where supported.
const moveEvent = 'onpointerrawupdate' in window ? 'pointerrawupdate' : 'pointermove';
addEventListener(moveEvent, (e) => {
  mouse.x = e.clientX;
  mouse.y = e.clientY;
  ensureFocus();
  request();
});

// ---------- boot

async function init() {
  const [raw, state] = await Promise.all([invoke('frame'), invoke('state'), document.fonts.load(`600 13px ${FONT}`)]);
  const buffer = raw instanceof ArrayBuffer ? raw : Uint8Array.from(raw).buffer;
  const [width, height, x, y] = new Int32Array(buffer, 0, 4);
  const data = new Uint8ClampedArray(buffer, 16);
  const image = new ImageData(data, width, height);

  canvas.width = width;
  canvas.height = height;
  ctx.putImageData(image, 0, 0);

  const scale = width / innerWidth;
  shot = { data, width, height, scale, bitmap: await createImageBitmap(image) };
  keys = state.settings;
  const theme = state.settings.theme;
  const light = theme === 'light' || (theme === 'system' && matchMedia('(prefers-color-scheme: light)').matches);
  hintColors = light ? HINT_COLORS.light : HINT_COLORS.dark;
  formatIndex = Math.min(state.settings.format, FORMATS.length - 1);
  hints = measureHints();
  mouse.x = x / scale;
  mouse.y = y / scale;

  draw();
  await invoke('ready');
}

init();
