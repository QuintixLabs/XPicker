// ============================================================
// controls.js
// Your keys and mouse in the picker: copy the color, switch the
// format, cancel, or open XPicker.
// ============================================================

import { FORMATS } from '../shared/color-formats.js';
import { fromKey, fromMouse } from '../shared/bindings.js';
import { pixelAt, pixelUnder, request, shot } from './screen.js';

const { invoke } = window.__TAURI__.core;

const CYCLE_MS = 240; // how fast the format changes while the key is held

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

// ------------------------------------------------------------
// Keyboard and mouse
// ------------------------------------------------------------

addEventListener('keydown', (e) => handle(fromKey(e), e));

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
  mouse.x = e.clientX;
  mouse.y = e.clientY;
  ensureFocus();
  request();
});
