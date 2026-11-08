// ============================================================
// color-formats.js
// Writes a color as HEX, RGB, RGBA, HSL or HSLA, and tells light
// colors from dark ones.
// ============================================================

export const FORMATS = [
  { name: 'HEX', format: (c) => `#${c.hex}` },
  { name: 'RGB', format: (c) => `rgb(${c.r},${c.g},${c.b})` },
  { name: 'RGBA', format: (c) => `rgba(${c.r},${c.g},${c.b},1)` },
  { name: 'HSL', format: (c) => { const { h, s, l } = toHsl(c); return `hsl(${h},${s}%,${l}%)`; } },
  { name: 'HSLA', format: (c) => { const { h, s, l } = toHsl(c); return `hsla(${h},${s}%,${l}%,1)`; } },
];

export function rgb(r, g, b) {
  const hex = [r, g, b].map((v) => v.toString(16).padStart(2, '0')).join('').toUpperCase();
  return { r, g, b, hex };
}

export function isLight({ r, g, b }) {
  return (r * 299 + g * 587 + b * 114) / 1000 > 128;
}

function toHsl({ r, g, b }) {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  let h = 0;
  let s = 0;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === r) h = (g - b) / d + (g < b ? 6 : 0);
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    h *= 60;
  }

  return { h: Math.round(h), s: Math.round(s * 100), l: Math.round(l * 100) };
}
