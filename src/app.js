// Main window — welcome screen, recent colors and settings.

import './lib/native.js';
import { FORMATS, rgb, isLight } from './lib/color.js';
import { fromKey, fromMouse, bindingKeys, shortcutKeys, shortcutFromKey } from './lib/keys.js';

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const $ = (sel) => document.querySelector(sel);
const $$ = (sel) => [...document.querySelectorAll(sel)];

const ACTIONS = { pick: 'Copy color', cycle: 'Switch format', cancel: 'Cancel', open: 'Open XPicker' };
const DEFAULTS = {
  shortcut: 'Ctrl+Shift+KeyX',
  pick: 'Mouse:2',
  cycle: 'Key:Control',
  cancel: 'Key:Escape',
  open: 'Key:Tab',
  notify: true,
  historySize: 10,
  startWithComputer: false,
  pickOnLaunch: false,
  closeToTray: true,
  autoUpdate: true,
};

let state = null;
let recording = null; // { el, field }

// ---------- helpers

const caps = (keys) => keys.map((k) => `<kbd>${k}</kbd>`).join('');
const format = () => FORMATS[state.settings.format] ?? FORMATS[0];

function fromHex(hex) {
  const n = parseInt(hex, 16);
  return rgb(n >> 16, (n >> 8) & 255, n & 255);
}

function ago(at) {
  const min = (Date.now() - at) / 60000;
  if (min < 1) return 'Just now';
  if (min < 60) return `${Math.floor(min)}m ago`;
  if (min < 1440) return `${Math.floor(min / 60)}h ago`;
  return `${Math.floor(min / 1440)}d ago`;
}

/** Small status line. The welcome screen only ever shows errors, never hints or "Saved". */
function status(text, error = false) {
  for (const el of $$('.status')) {
    const quiet = el.closest('.welcome') && !error;
    el.textContent = quiet ? '' : (text ?? el.dataset.default ?? '');
    el.classList.toggle('is-error', error);
  }
}

async function save(patch) {
  try {
    state = await invoke('save_settings', { settings: { ...state.settings, ...patch } });
    status();
    render();
    return true;
  } catch (err) {
    status(String(err), true);
    return false;
  }
}

async function copy(hex, label) {
  await invoke('copy', { text: format().format(fromHex(hex)) });
  const text = label.textContent;
  label.textContent = 'Copied';
  setTimeout(() => (label.textContent = text), 1100);
}

function confirmAction({ title, text, action }) {
  const dialog = $('#confirm');
  $('#confirm-title').textContent = title;
  $('#confirm-text').textContent = text;
  $('#confirm-ok').textContent = action;
  dialog.returnValue = '';
  dialog.classList.remove('is-closing');
  dialog.showModal();
  document.activeElement?.blur(); // no pre-focused button ring on open
  return new Promise((resolve) => {
    dialog.addEventListener('close', () => resolve(dialog.returnValue === 'ok'), { once: true });
  });
}

/** Plays the fade-out, then actually closes (buttons and Esc both come through here). */
function closeDialog(dialog, value) {
  if (dialog.classList.contains('is-closing')) return;
  dialog.classList.add('is-closing');
  dialog.addEventListener('animationend', () => dialog.close(value), { once: true });
}

$('#confirm form').addEventListener('submit', (e) => {
  e.preventDefault();
  closeDialog($('#confirm'), e.submitter?.value ?? '');
});

$('#confirm').addEventListener('cancel', (e) => {
  e.preventDefault();
  closeDialog($('#confirm'), '');
});

// ---------- sidebar (collapsed state is a per-machine preference; lib/boot.js
// restores it before the first paint)

const SIDEBAR_KEY = 'xpicker.sidebarCollapsed';
const root = document.documentElement;
const isCollapsed = () => root.classList.contains('is-collapsed');
const updateToggleTitle = () => ($('#sidebar-toggle').title = isCollapsed() ? 'Expand sidebar' : 'Collapse sidebar');

updateToggleTitle();
// Animate only user toggles, not the restored state on open.
requestAnimationFrame(() => requestAnimationFrame(() => root.classList.add('can-animate')));

$('#sidebar-toggle').addEventListener('click', () => {
  const collapsed = !isCollapsed();
  root.classList.toggle('is-collapsed', collapsed);
  updateToggleTitle();
  try {
    localStorage.setItem(SIDEBAR_KEY, collapsed ? '1' : '0');
  } catch {}

  // The cursor is still over the brand right after toggling; ignore that hover
  // until it leaves, or the logo/toggle swap would flicker.
  root.classList.add('hover-lock');
  $('.brand').addEventListener('mouseleave', () => root.classList.remove('hover-lock'), { once: true });
});

// ---------- views

function show(view) {
  stopRecording();
  status();
  document.body.classList.toggle('is-welcome', view === 'welcome');
  $$('.view').forEach((el) => (el.hidden = el.id !== view));
  $$('[data-tab]').forEach((tab) => tab.classList.toggle('is-active', tab.dataset.tab === view));
  $('main').scrollTop = 0;
}

for (const tab of $$('[data-tab]')) {
  tab.addEventListener('click', () => show(tab.dataset.tab));
}

// ---------- welcome steps

const NEXT_LABELS = ['Get started', 'Next', 'Start using XPicker'];
let step = 0;

function goStep(n) {
  stopRecording();
  status();
  step = n;
  $('#welcome').classList.toggle('is-intro', n === 0);
  $$('.step').forEach((el) => (el.hidden = Number(el.dataset.step) !== n));
  $$('.dots span').forEach((dot, i) => dot.classList.toggle('is-active', i === n));
  $('#back').hidden = n === 0;
  $('#next').textContent = NEXT_LABELS[n];
}

$('#back').addEventListener('click', () => goStep(step - 1));

$('#next').addEventListener('click', async () => {
  if (step < NEXT_LABELS.length - 1) return goStep(step + 1);
  if (await save({ onboarded: true })) show('colors');
});

$('#pick').addEventListener('click', () => invoke('start_pick'));

// ---------- colors

function renderColors() {
  const [latest, ...rest] = state.history;

  $('#formats').innerHTML = FORMATS.map(
    (f, i) => `<button class="${i === state.settings.format ? 'is-active' : ''}" data-format="${i}">${f.name}</button>`,
  ).join('');
  $('#clear').hidden = !latest;

  const hero = $('#hero');
  hero.classList.toggle('is-empty', !latest);
  if (latest) {
    const color = fromHex(latest.hex);
    hero.dataset.hex = latest.hex;
    hero.classList.toggle('is-dark', color.r + color.g + color.b < 60);
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
      <span class="hero-empty-hint">Press ${caps(shortcutKeys(state.settings.shortcut))} or <b>Pick color</b> to grab one.</span>`;
  }

  $('#earlier').hidden = rest.length === 0;
  $('#history').innerHTML = rest
    .map((entry) => {
      const color = fromHex(entry.hex);
      const dark = color.r + color.g + color.b < 60 ? ' is-dark' : '';
      return `
        <li>
          <button class="entry" data-hex="${entry.hex}">
            <span class="swatch${dark}" style="--color:#${entry.hex}"></span>
            <span class="entry-value">${format().format(color)}</span>
            <span class="entry-time">${ago(entry.at)}</span>
          </button>
        </li>`;
    })
    .join('');
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
  const count = state.history.length;
  const ok = await confirmAction({
    title: 'Clear all colors?',
    text: `This deletes ${count} saved color${count === 1 ? '' : 's'}. You can't undo this.`,
    action: 'Clear all',
  });
  if (!ok) return;
  state = await invoke('clear_history');
  render();
});

// ---------- settings

function renderSettings() {
  for (const el of $$('.recorder')) {
    if (el === recording?.el) continue;
    const value = state.settings[el.dataset.field];
    el.innerHTML = caps(el.dataset.field === 'shortcut' ? shortcutKeys(value) : bindingKeys(value));
  }

  for (const el of $$('.segmented[data-field]')) {
    const current = String(state.settings[el.dataset.field]);
    el.innerHTML = el.dataset.options
      .split(',')
      .map((option) => {
        const [value, label] = option.split(':');
        return `<button class="${value === current ? 'is-active' : ''}" data-value="${value}">${label}</button>`;
      })
      .join('');
  }

  for (const el of $$('.toggle')) {
    el.setAttribute('aria-checked', String(Boolean(state.settings[el.dataset.field])));
  }

  for (const el of $$('[data-caps]')) {
    el.innerHTML = caps(bindingKeys(state.settings[el.dataset.caps]));
  }

  for (const el of $$('[data-shortcut]')) {
    el.innerHTML = caps(shortcutKeys(state.settings.shortcut));
  }

  $('#pick-keys').innerHTML = caps(shortcutKeys(state.settings.shortcut));
}

for (const el of $$('.segmented[data-field]')) {
  el.addEventListener('click', (e) => {
    const button = e.target.closest('[data-value]');
    if (button) save({ [el.dataset.field]: Number(button.dataset.value) });
  });
}

for (const el of $$('.toggle')) {
  el.addEventListener('click', () => save({ [el.dataset.field]: !state.settings[el.dataset.field] }));
}

$('#reset').addEventListener('click', async () => {
  const ok = await confirmAction({
    title: 'Reset settings?',
    text: 'Shortcuts and preferences go back to their defaults. Your colors are kept.',
    action: 'Reset',
  });
  if (ok) save(DEFAULTS);
});

$('#replay').addEventListener('click', () => {
  show('welcome');
  goStep(0);
});

// ---------- updates + footer links

let appVersion = '';
window.__TAURI__.app.getVersion().then((v) => {
  appVersion = v;
  $('#version').textContent = `Version ${v}`;
});

// Automatic updates report what they're doing ("Downloading XPicker 1.0.2…").
listen('update-status', (e) => {
  const text = e.payload === 'XPicker' ? '' : ` · ${e.payload}`;
  $('#version').textContent = `Version ${appVersion}${text}`;
});

for (const button of $$('[data-link]')) {
  button.addEventListener('click', () => invoke('open_link', { name: button.dataset.link }));
}

const UPDATE_TEXT = {
  idle: 'Check for updates',
  checking: 'Checking…',
  latest: "You're up to date",
  updating: 'Updating…',
  error: "Couldn't check for updates",
};
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
let loadingSvg = '';
fetch('assets/icons/loading.svg').then((r) => r.text()).then((svg) => (loadingSvg = svg));

/** Cross-fades the update button to a new state. */
async function setUpdate(state, text = UPDATE_TEXT[state]) {
  const button = $('#update');
  const content = button.querySelector('.update-content');
  content.classList.add('is-swapping');
  await wait(140);
  button.dataset.state = state;
  const icon = { checking: loadingSvg, updating: loadingSvg, latest: '<i class="icon icon-check"></i>' }[state] ?? '';
  button.querySelector('.update-icon').innerHTML = icon;
  button.querySelector('.update-text').textContent = text;
  content.classList.remove('is-swapping');
}

async function checkForUpdates() {
  if ($('#update').dataset.state !== 'idle') return;
  await setUpdate('checking');
  const started = performance.now();
  try {
    const version = await invoke('check_update');
    await wait(Math.max(0, 800 - (performance.now() - started))); // let the spinner read as a check
    if (!version) {
      await setUpdate('latest');
      await wait(3500);
      return setUpdate('idle');
    }
    await setUpdate('updating', `Updating to v${version}…`);
    await invoke('install_update'); // restarts into the new version when done
  } catch {
    await setUpdate('error');
    await wait(4000);
    setUpdate('idle');
  }
}

$('#update').addEventListener('click', checkForUpdates);
listen('update-progress', (e) => {
  $('#update .update-text').textContent = `Updating… ${e.payload}%`;
});

// ---------- shortcut recording

function startRecording(el) {
  stopRecording();
  recording = { el, field: el.dataset.field };
  el.classList.add('is-recording');
  el.innerHTML = `<span class="hint">${recording.field === 'shortcut' ? 'Press a shortcut…' : 'Press a key or click…'}</span>`;
  status(recording.field === 'shortcut' ? 'Hold Ctrl, Alt or Shift and press a key. Esc to stop.' : 'Any key or mouse button works. Esc to stop.');
  if (recording.field === 'shortcut') invoke('pause_shortcut', { paused: true });
}

function stopRecording() {
  if (!recording) return;
  const { el, field } = recording;
  recording = null;
  el.classList.remove('is-recording');
  if (field === 'shortcut') invoke('pause_shortcut', { paused: false });
  renderSettings();
}

async function commit(field, value) {
  const clash = Object.keys(ACTIONS).find((f) => f !== field && state.settings[f] === value);
  if (clash) {
    status(`${bindingKeys(value)[0]} is already used for ${ACTIONS[clash]}. Try another.`, true);
    return;
  }
  const { el } = recording;
  recording = null; // stop listening while saving
  el.classList.remove('is-recording');
  const saved = await save({ [field]: value });
  if (field === 'shortcut') invoke('pause_shortcut', { paused: false });
  if (saved) status();
  renderSettings();
}

// mousedown (not click) so binding "Left click" here doesn't immediately re-arm it.
for (const el of $$('.recorder')) {
  el.addEventListener('mousedown', (e) => {
    if (e.button === 0 && !recording) startRecording(el);
  });
}

addEventListener(
  'keydown',
  (e) => {
    if (!recording || e.repeat) return;
    e.preventDefault();
    e.stopPropagation();
    const { el, field } = recording;
    const bare = !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey;

    if (field === 'shortcut') {
      if (e.code === 'Escape' && bare) return stopRecording();
      const { mods, shortcut } = shortcutFromKey(e);
      if (shortcut) return commit(field, shortcut);
      el.innerHTML = mods.length
        ? `${caps(shortcutKeys(mods.join('+')))}<span class="hint">+ key</span>`
        : '<span class="hint">Add Ctrl, Alt or Shift</span>';
      return;
    }

    if (e.code === 'Escape' && field !== 'cancel') return stopRecording();
    commit(field, fromKey(e));
  },
  true,
);

addEventListener(
  'mousedown',
  (e) => {
    if (!recording) return;
    const onRecorder = e.target.closest('.recorder') === recording.el;
    if (!onRecorder) return stopRecording();
    e.preventDefault();
    e.stopPropagation();
    if (recording.field !== 'shortcut') commit(recording.field, fromMouse(e));
  },
  true,
);

// Clicked buttons don't keep focus, so no focus ring is left behind on them.
addEventListener('mouseup', (e) => e.target.closest('button')?.blur());

// Side buttons would otherwise navigate the webview; right click opens a menu.
for (const type of ['mouseup', 'auxclick', 'contextmenu']) {
  addEventListener(type, (e) => e.preventDefault(), true);
}

// ---------- boot

function render() {
  renderColors();
  renderSettings();
}

listen('state', (e) => {
  state = e.payload;
  render();
});

state = await invoke('state');
render();
show(state.settings.onboarded ? 'colors' : 'welcome');
goStep(0);
setInterval(renderColors, 60000); // keep "5m ago" fresh
