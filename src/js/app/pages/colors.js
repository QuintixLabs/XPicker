// ============================================================
// colors.js
// The Colors page: your latest pick, the list under it, and
// copying them in the format you choose.
// ============================================================

import { FORMATS, rgb, isLight } from '../../shared/color-formats.js';
import { keycaps, shortcutKeys } from '../../shared/bindings.js';
import { $, app, invoke, onRender, render, save } from '../state.js';
import { confirmAction } from '../components/dialog.js';

const format = () => FORMATS[app.state.settings.format] ?? FORMATS[0];

function fromHex(hex) {
  const n = parseInt(hex, 16);
  return rgb(n >> 16, (n >> 8) & 255, n & 255);
}

const isDark = (color) => color.r + color.g + color.b < 60;

function ago(at) {
  const min = (Date.now() - at) / 60000;
  if (min < 1) return 'Just now';
  if (min < 60) return `${Math.floor(min)}m ago`;
  if (min < 1440) return `${Math.floor(min / 60)}h ago`;
  return `${Math.floor(min / 1440)}d ago`;
}

// ------------------------------------------------------------
// Drawing the page
// ------------------------------------------------------------

function renderColors(state) {
  // All recent colors are kept; "Recent colors" only decides how many are shown.
  const [latest, ...rest] = state.history.slice(0, state.settings.historySize);

  $('#formats').innerHTML = FORMATS.map(
    (f, i) => `<button class="${i === state.settings.format ? 'is-active' : ''}" data-format="${i}">${f.name}</button>`,
  ).join('');
  $('#clear').hidden = !latest;

  const hero = $('#hero');
  hero.classList.toggle('is-empty', !latest);
  if (latest) {
    const color = fromHex(latest.hex);
    hero.dataset.hex = latest.hex;
    hero.classList.toggle('is-dark', isDark(color));
    hero.style.setProperty('--color', `#${latest.hex}`);
    hero.style.color = isLight(color) ? '#000' : '#fff';
    hero.innerHTML = `
      <span class="hero-hint">Click to copy</span>
      <span class="hero-value">${format().format(color)}</span>
      <span class="hero-meta">Latest · ${ago(latest.at)}</span>`;
  } else {
    delete hero.dataset.hex;
    hero.style.removeProperty('color');
    hero.innerHTML = `
      <span class="hero-empty-title">No colors yet</span>
      <span class="hero-empty-hint">Press ${keycaps(shortcutKeys(state.settings.shortcut))} or <b>Pick color</b> to grab one.</span>`;
  }

  $('#earlier').hidden = rest.length === 0;
  $('#history').innerHTML = rest
    .map((entry) => {
      const color = fromHex(entry.hex);
      return `
        <li>
          <button class="entry" data-hex="${entry.hex}">
            <span class="swatch${isDark(color) ? ' is-dark' : ''}" style="--color:#${entry.hex}"></span>
            <span class="entry-value">${format().format(color)}</span>
            <span class="entry-time">${ago(entry.at)}</span>
          </button>
        </li>`;
    })
    .join('');
}

onRender(renderColors);
setInterval(() => app.state && renderColors(app.state), 60000); // keep "5m ago" fresh

// ------------------------------------------------------------
// Copying and clearing
// ------------------------------------------------------------

/** Copies, shows "Copied" for a moment, then puts the real value back, however fast you click. */
async function copy(hex, label) {
  await invoke('copy', { text: format().format(fromHex(hex)) });
  label.textContent = 'Copied';
  clearTimeout(label.copiedTimer);
  label.copiedTimer = setTimeout(() => (label.textContent = format().format(fromHex(hex))), 1100);
}

$('#formats').addEventListener('click', (e) => {
  const button = e.target.closest('[data-format]');
  if (button) save({ format: Number(button.dataset.format) });
});

$('#hero').addEventListener('click', (e) => {
  const hero = e.currentTarget;
  if (hero.dataset.hex) copy(hero.dataset.hex, hero.querySelector('.hero-value'));
});

$('#history').addEventListener('click', (e) => {
  const entry = e.target.closest('.entry');
  if (entry) copy(entry.dataset.hex, entry.querySelector('.entry-value'));
});

$('#clear').addEventListener('click', async () => {
  const count = app.state.history.length;
  const ok = await confirmAction({
    title: 'Clear all colors?',
    text: `This deletes ${count} saved color${count === 1 ? '' : 's'}. You can't undo this.`,
    action: 'Clear all',
  });
  if (!ok) return;
  app.state = await invoke('clear_history');
  render();
});
