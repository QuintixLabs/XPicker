// ============================================================
// controls.js
// Your keys and mouse in the picker: copy the color, switch the
// format, cancel, or open XPicker.
// ============================================================

import { FORMATS } from '../shared/color-formats.js';
import { fromKey, fromMouse } from '../shared/bindings.js';
import { LINUX, pixelAt, pixelUnder, request, shot } from './screen.js';

const { invoke } = window.__TAURI__.core;

const CYCLE_MS = 240; // how fast the format changes while the key is held

// Arrow keys move one pixel, with Shift ten. A key you've bound to an action
// in Settings does that action instead.
const ARROWS = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };
const BIG_STEP = 10;

// Linux can't move the real cursor, so arrows only move XPicker's crosshair.
// How far they moved it is kept, so the mouse carries on from there instead of
// jumping back to the real cursor.
let real = null;              // where the real cursor is, in css px
let offset = { x: 0, y: 0 };  // crosshair minus real cursor

/** Where the mouse is, in css px. */
export const mouse = { x: 0, y: 0 };

let keys = {};
let formatIndex = 0;
let lastCycle = 0;
let done = false;     // one pick, cancel or open per picker
let armed = null;     // mouse button pressed down, acted on when released

/** Sets up your keys and the format you used last. */
export function prepare(settings) {
  keys = settings;
  formatIndex = Math.min(settings.format, FORMATS.length - 1);
  real = { ...mouse };
  offset = { x: 0, y: 0 };
}

export const currentFormat = () => formatIndex;

// ------------------------------------------------------------
// Actions
// ------------------------------------------------------------

async function finish(command, args) {
  if (done) return;
  done = true;
  invoke(command, args);
}

function pick() {
  const { x, y } = pixelUnder(mouse);
  const color = pixelAt(x, y);
  finish('pick', { value: FORMATS[formatIndex].format(color), hex: color.hex, format: formatIndex });
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
    [keys.cancel]: () => finish('dismiss'),
    [keys.open]: () => finish('open_from_picker'),
  }[binding];
}

function handle(binding, e) {
  if (!shot.loaded || done) return;
  const action = actionFor(binding, e);
  if (!action) return;
  e.preventDefault();
  action();
}

/** Moves the picked spot by whole screen pixels, and the mouse cursor with it. */
function nudge([dx, dy], step) {
  const here = pixelUnder(mouse);
  const x = Math.min(Math.max(here.x + dx * step, 0), shot.width - 1);
  const y = Math.min(Math.max(here.y + dy * step, 0), shot.height - 1);
  mouse.x = (x + 0.5) / shot.scale;
  mouse.y = (y + 0.5) / shot.scale;
  if (LINUX) offset = { x: mouse.x - real.x, y: mouse.y - real.y };
  else invoke('move_cursor', { x, y });
  request();
}

/** The real cursor moved: on Linux keep the arrows' offset, shrinking it at the screen edges. */
function follow(x, y) {
  real = { x, y };
  const maxX = shot.cssWidth - 0.001;
  const maxY = shot.cssHeight - 0.001;
  mouse.x = Math.min(Math.max(x + offset.x, 0), maxX);
  mouse.y = Math.min(Math.max(y + offset.y, 0), maxY);
  // Pushed against an edge: what didn't fit comes off the offset, so every
  // pixel stays reachable with the mouse.
  offset = { x: mouse.x - x, y: mouse.y - y };
}

// ------------------------------------------------------------
// Keyboard and mouse
// ------------------------------------------------------------

addEventListener('keydown', (e) => {
  const binding = fromKey(e);
  if (actionFor(binding, e)) return handle(binding, e);
  if (ARROWS[e.code] && shot.loaded && !done) {
    e.preventDefault();
    nudge(ARROWS[e.code], e.shiftKey ? BIG_STEP : 1);
  }
});

// Mouse buttons act on release: if the picker closed on press, the release
// would land on the app underneath (a right-click there opens its menu).
addEventListener('mousedown', (e) => {
  e.preventDefault();
  const binding = fromMouse(e);
  if (shot.loaded && actionFor(binding, e)) armed = binding;
});
addEventListener('mouseup', (e) => {
  e.preventDefault();
  const binding = fromMouse(e);
  if (binding !== armed) return;
  armed = null;
  handle(binding, e);
});

// Windows sometimes refuses the first focus request; keep asking so keys
// (Esc especially) always reach the picker.
let lastFocusTry = 0;
function ensureFocus() {
  const now = performance.now();
  if (document.hasFocus() || now - lastFocusTry < 400) return;
  lastFocusTry = now;
  invoke('ready');
}

// pointerrawupdate follows the mouse at its own speed where supported.
const moveEvent = 'onpointerrawupdate' in window ? 'pointerrawupdate' : 'pointermove';
addEventListener(moveEvent, (e) => {
  if (!shot.loaded || done) return;
  follow(e.clientX, e.clientY);
  ensureFocus();
  request();
});
