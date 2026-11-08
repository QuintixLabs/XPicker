// ============================================================
// welcome.js
// The welcome screen on first launch: hello, set up your keys,
// you're all set.
// ============================================================

import { bindingKeys, keycaps, shortcutKeys } from '../../shared/bindings.js';
import { $, $$, onRender, save, status } from '../state.js';
import { stopRecording } from '../components/key-recorder.js';
import { show } from '../components/sidebar.js';

const NEXT_LABELS = ['Get started', 'Next', 'Start using XPicker'];
let step = 0;

/** Shows step 0, 1 or 2. */
export function goStep(n) {
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
  if (await save({ showWelcome: false })) show('colors');
});

// "You're all set" lists the same keys, read-only.
onRender((state) => {
  for (const el of $$('[data-caps]')) el.innerHTML = keycaps(bindingKeys(state.settings[el.dataset.caps]));
  for (const el of $$('[data-shortcut]')) el.innerHTML = keycaps(shortcutKeys(state.settings.shortcut));
});
