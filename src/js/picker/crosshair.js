// ============================================================
// picker/crosshair.js
// Linux: XPicker's own crosshair, so it moves with the loupe.
// ============================================================

// On Linux the system cursor moves before XPicker hears about it, so on a fast
// flick it could land on the loupe. Drawing the crosshair ourselves keeps it
// exactly where the picker is, on the pixel that gets picked.

import { LINUX, moveLayer, shot, sizeLayer, snap } from './screen.js';

const ARM = 10;   // length of each line, in css px
const GAP = 3;    // empty space around the picked pixel, so you can see it
const THICK = 1;  // line thickness, in css px
const REACH = ARM + GAP + 3; // half the layer's size

const layer = document.getElementById('crosshair');
let g = null;

/** Called once the screenshot is loaded. */
export function prepare() {
  if (!LINUX) return;
  document.documentElement.classList.add('own-cursor');
  g = sizeLayer(layer, REACH * 2 + 1, REACH * 2 + 1);
  layer.hidden = false;
}

/** Puts the crosshair on screenshot pixel `px, py`. */
export function draw(px, py) {
  if (!LINUX) return;
  const s = shot.scale;
  // The picked pixel's top left, in css px, and the layer's corner around it.
  const left = px / s;
  const top = py / s;
  const ox = snap(left - REACH);
  const oy = snap(top - REACH);

  // Draw in screen pixels so the lines are crisp: the picked pixel is (cx, cy).
  g.setTransform(1, 0, 0, 1, 0, 0);
  g.clearRect(0, 0, layer.width, layer.height);
  const cx = Math.round((left - ox) * s);
  const cy = Math.round((top - oy) * s);
  const arm = Math.round(ARM * s);
  const gap = Math.round(GAP * s);
  // Thickness spread evenly around the picked pixel's row and column.
  const t = Math.max(1, Math.round(THICK * s));
  const lo = Math.floor((t - 1) / 2);

  // Each line: a dark edge, then white on top, so it shows on any color.
  const lines = [
    [cx - gap - arm, cy - lo, arm, t], // left
    [cx + 1 + gap, cy - lo, arm, t],   // right
    [cx - lo, cy - gap - arm, t, arm], // up
    [cx - lo, cy + 1 + gap, t, arm],   // down
  ];
  g.fillStyle = 'rgba(0, 0, 0, 0.75)';
  for (const [x, y, w, h] of lines) g.fillRect(x - 1, y - 1, w + 2, h + 2);
  g.fillStyle = '#fff';
  for (const [x, y, w, h] of lines) g.fillRect(x, y, w, h);

  moveLayer(layer, ox, oy);
}
