// ============================================================
// controls/key-recorder.js
// The key boxes in Settings and the welcome screen: click one,
// then press a key or mouse button to change it.
// ============================================================

import { fromKey, fromMouse, bindingKeys, shortcutKeys, shortcutFromKey, keycaps } from '../core/bindings.js';
import { $$, ACTIONS, app, invoke, onRender, save, status } from '../core/state.js';

let recording = null; // { el, field } while a box is listening

// ------------------------------------------------------------
// Showing the keys
// ------------------------------------------------------------

function renderBoxes() {
  for (const el of $$('.recorder')) {
    if (el === recording?.el) continue;
    const value = app.state.settings[el.dataset.field];
    el.innerHTML = keycaps(el.dataset.field === 'shortcut' ? shortcutKeys(value) : bindingKeys(value));
  }
}

onRender(renderBoxes);

const listeningHint = (field) => (field === 'shortcut' ? 'Press a shortcut…' : 'Press a key or click…');

// ------------------------------------------------------------
// Listening for a new key
// ------------------------------------------------------------

function start(el) {
  stopRecording();
  recording = { el, field: el.dataset.field };
  el.classList.add('is-recording');
  el.innerHTML = `<span class="hint">${listeningHint(recording.field)}</span>`;
  // The old shortcut is let go while recording, so pressing it doesn't open the picker.
  if (recording.field === 'shortcut') invoke('pause_shortcut', { paused: true });
}

/** Stops listening without changing anything. */
export function stopRecording() {
  if (!recording) return;
  const { el, field } = recording;
  recording = null;
  el.classList.remove('is-recording', 'is-clash');
  clearTimeout(el.clashTimer);
  if (field === 'shortcut') invoke('pause_shortcut', { paused: false });
  renderBoxes();
}

/** A key that's already taken: say so inside the box, keep listening, and light up its owner. */
function showClash(el, owner) {
  el.classList.add('is-clash');
  el.innerHTML = `<span class="hint">Used by ${ACTIONS[owner]}</span>`;
  for (const box of $$(`.recorder[data-field="${owner}"]`)) {
    box.classList.add('is-flash');
    setTimeout(() => box.classList.remove('is-flash'), 1200);
  }
  clearTimeout(el.clashTimer);
  el.clashTimer = setTimeout(() => {
    if (recording?.el !== el) return;
    el.classList.remove('is-clash');
    el.innerHTML = `<span class="hint">${listeningHint(recording.field)}</span>`;
  }, 2000);
}

async function commit(field, value) {
  const owner = Object.keys(ACTIONS).find((f) => f !== field && app.state.settings[f] === value);
  if (owner) return showClash(recording.el, owner); // never saved; still listening

  const { el } = recording;
  clearTimeout(el.clashTimer);
  el.classList.remove('is-clash', 'is-recording');
  recording = null;
  const saved = await save({ [field]: value });
  if (field === 'shortcut') invoke('pause_shortcut', { paused: false });
  if (saved) status();
  renderBoxes();
}

// ------------------------------------------------------------
// Keyboard and mouse
// ------------------------------------------------------------

// mousedown (not click), so binding "Left click" here doesn't start listening again.
for (const el of $$('.recorder')) {
  el.addEventListener('mousedown', (e) => {
    if (e.button === 0 && !recording) start(el);
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
        ? `${keycaps(shortcutKeys(mods.join('+')))}<span class="hint">+ key</span>`
        : '<span class="hint">Add Ctrl, Alt or Shift</span>';
      return;
    }

    if (e.code === 'Escape' && field !== 'cancel') return stopRecording();
    commit(field, fromKey(e));
  },
  true,
);

// While a box is listening: right, middle and side buttons count anywhere in the
// window; a left click counts inside the box and cancels outside it.
addEventListener(
  'mousedown',
  (e) => {
    if (!recording) return;
    const onBox = e.target.closest('.recorder') === recording.el;
    if (!onBox && e.button === 0) return stopRecording();
    e.preventDefault();
    e.stopPropagation();
    if (recording.field === 'shortcut') {
      if (!onBox) stopRecording(); // the start shortcut is keys only
      return;
    }
    commit(recording.field, fromMouse(e));
  },
  true,
);
