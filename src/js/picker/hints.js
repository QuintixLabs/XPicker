// ============================================================
// hints.js
// The bar that shows your keys while picking. When the loupe
// reaches it, it fades over to the top of the screen, and back.
// ============================================================

import { bindingKeys } from '../shared/bindings.js';
import { FONT } from './loupe.js';
import { ctx, markDirty, roundRect, shot } from './screen.js';

const HEIGHT = 46;
const MARGIN = 24;           // from the screen edge
const KEY_FONT = `600 12px ${FONT}`;
const LABEL_FONT = `500 14px ${FONT}`;
const FADE_MS = 110;         // each half of the fade: out at one place, in at the other

// Follows the app's theme ("system" = the computer's setting).
const COLORS = {
  dark: { bg: 'rgba(14, 14, 16, 0.9)', border: 'rgba(255, 255, 255, 0.08)', cap: 'rgba(255, 255, 255, 0.1)', key: '#f4f4f5', label: '#a1a1a8' },
  light: { bg: 'rgba(255, 255, 255, 0.94)', border: 'rgba(0, 0, 0, 0.1)', cap: 'rgba(0, 0, 0, 0.06)', key: '#18181b', label: '#6b6b74' },
};

let colors = COLORS.dark;
let layout = null;           // measured keys and labels
let place = 'bottom';        // where it is (or is fading to)
let from = 'bottom';         // where it was before the last switch
let switchedAt = -Infinity;

// ------------------------------------------------------------
// Setting up
// ------------------------------------------------------------

/** Measures the bar for these keys and picks its colors. Called at the start of each pick. */
export function prepare(settings) {
  const light = settings.theme === 'light' || (settings.theme === 'system' && matchMedia('(prefers-color-scheme: light)').matches);
  colors = light ? COLORS.light : COLORS.dark;
  place = from = 'bottom';
  switchedAt = -Infinity;

  const items = [
    [bindingKeys(settings.pick), 'Copy'],
    [bindingKeys(settings.cycle), 'Format'],
    [bindingKeys(settings.cancel), 'Cancel'],
    [bindingKeys(settings.open), 'Open XPicker'],
  ].map(([caps, label]) => {
    ctx.font = KEY_FONT;
    const capWidths = caps.map((c) => ctx.measureText(c).width + 16);
    ctx.font = LABEL_FONT;
    const width = capWidths.reduce((a, b) => a + b + 4, -4) + 8 + ctx.measureText(label).width;
    return { caps, capWidths, label, width };
  });
  layout = { items, width: items.reduce((a, item) => a + item.width + 24, -24) + 36 };
}

// ------------------------------------------------------------
// Where it goes
// ------------------------------------------------------------

const overlaps = (a, b) => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
const contains = (r, p) => p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h;

function rectAt(where, grow = 0) {
  const y = where === 'top' ? MARGIN : shot.cssHeight - MARGIN - HEIGHT;
  return { x: (shot.cssWidth - layout.width) / 2 - grow, y: y - grow, w: layout.width + grow * 2, h: HEIGHT + grow * 2 };
}

/**
 * Leaves the bottom when the loupe reaches it. Comes back only once both the
 * loupe and the cursor have clearly left the bottom area, so the loupe flipping
 * up near the edge can't send it back and forth.
 */
function wanted(loupe, cursor) {
  if (place === 'bottom') return overlaps(rectAt('bottom', 12), loupe) ? 'top' : 'bottom';
  const bottom = rectAt('bottom', 48);
  return overlaps(bottom, loupe) || contains(bottom, cursor) ? 'top' : 'bottom';
}

// ------------------------------------------------------------
// Drawing
// ------------------------------------------------------------

const smooth = (t) => t * t * (3 - 2 * t);

/** Draws the bar. Returns true while it's still fading, so the picker keeps drawing. */
export function draw(loupe, cursor) {
  const now = performance.now();
  const next = wanted(loupe, cursor);
  if (next !== place) {
    from = place;
    place = next;
    switchedAt = now;
  }

  const t = now - switchedAt;
  if (t < FADE_MS) {
    drawAt(from, 1 - smooth(t / FADE_MS));
  } else if (t < FADE_MS * 2) {
    drawAt(place, smooth((t - FADE_MS) / FADE_MS));
  } else {
    drawAt(place, 1);
  }
  return t < FADE_MS * 2;
}

function drawAt(where, alpha) {
  const { x, y } = rectAt(where);
  const left = Math.round(x);
  const top = Math.round(y);

  ctx.globalAlpha = alpha;
  ctx.fillStyle = colors.bg;
  roundRect(left, top, layout.width, HEIGHT, 12);
  ctx.fill();
  ctx.strokeStyle = colors.border;
  ctx.lineWidth = 1;
  ctx.stroke();

  ctx.textBaseline = 'middle';
  ctx.textAlign = 'left';
  let cursor = left + 18;
  const mid = top + HEIGHT / 2 + 1;
  for (const item of layout.items) {
    item.caps.forEach((cap, i) => {
      const w = item.capWidths[i];
      ctx.fillStyle = colors.cap;
      roundRect(cursor, top + 9, w, HEIGHT - 18, 6);
      ctx.fill();
      ctx.fillStyle = colors.key;
      ctx.font = KEY_FONT;
      ctx.fillText(cap, cursor + 8, mid);
      cursor += w + 4;
    });
    cursor += 4;
    ctx.fillStyle = colors.label;
    ctx.font = LABEL_FONT;
    ctx.fillText(item.label, cursor, mid);
    cursor += ctx.measureText(item.label).width + 24;
  }
  ctx.globalAlpha = 1;

  markDirty(left - 1, top - 1, layout.width + 2, HEIGHT + 2);
}
