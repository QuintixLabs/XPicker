// ============================================================
// pages/welcome.js
// The welcome screen: hello, your keys, all set.
// ============================================================

import { bindingKeys, keycaps, shortcutKeys } from '../core/bindings.js';
import { $, $$, onRender, save, status } from '../core/state.js';
import { stopRecording } from '../controls/key-recorder.js';
import { show } from '../window/sidebar.js';

const NEXT_LABELS = ['Get started', 'Next', 'Start using XPicker'];
const SLIDE = 12;       // px the steps slide while fading
const OUT_MS = 100;     // the old step fades out...
const IN_MS = 180;      // ...then the new one fades in
const EASE_OUT = 'cubic-bezier(0.16, 1, 0.3, 1)';

let step = 0;
let turn = 0;        // which change is current
let changingUntil = 0; // Next and Back ignore clicks until then, so a double-click can't skip a step

const stepEl = (n) => $(`.step[data-step="${n}"]`);
const shownStep = () => $$('.step').find((el) => !el.hidden);

/** Puts step `n` on screen straight away: its layout, buttons and dots. */
function apply(n) {
  $('#welcome').classList.toggle('is-intro', n === 0);
  for (const el of $$('.step')) {
    el.getAnimations().forEach((a) => a.cancel());
    el.inert = false;
    el.hidden = el !== stepEl(n);
  }
  $$('.dots span').forEach((dot, i) => dot.classList.toggle('is-active', i === n));
  $('#back').tabIndex = n === 0 ? -1 : 0;
  $('#next').textContent = NEXT_LABELS[n];
}

/**
 * Shows step 0, 1 or 2. The old step quickly fades out, then the new one fades
 * in, sliding a little in the direction you're going. The two never show
 * together. Between the hello step and the next one the buttons and dots sit
 * in a different place, so they fade out and back in with the step instead
 * of jumping.
 */
export function goStep(n) {
  stopRecording();
  status();
  const from = step;
  step = n;
  const mine = ++turn;

  const outgoing = shownStep();
  const incoming = stepEl(n);
  const controls = [$('.step-actions'), $('.dots')];
  controls.forEach((el) => el.getAnimations().forEach((a) => a.cancel()));
  if (!outgoing || outgoing === incoming || $('#welcome').hidden) {
    apply(n);
    return;
  }

  changingUntil = performance.now() + OUT_MS + IN_MS;
  const dir = n > from ? 1 : -1;
  const relayout = (from === 0) !== (n === 0);
  const fade = (el, keyframes, options) => el.animate(keyframes, options);
  const outOptions = { duration: OUT_MS, easing: 'ease-in', fill: 'forwards' };

  outgoing.getAnimations().forEach((a) => a.cancel());
  outgoing.inert = true;
  if (relayout) controls.forEach((el) => fade(el, [{ opacity: 1 }, { opacity: 0 }], outOptions));
  fade(outgoing, [{ opacity: 1, transform: 'none' }, { opacity: 0, transform: `translateX(${-SLIDE * dir}px)` }], outOptions)
    .finished.then(() => {
      if (mine !== turn) return; // another click took over
      controls.forEach((el) => el.getAnimations().forEach((a) => a.cancel()));
      apply(n);
      const inOptions = { duration: IN_MS, easing: EASE_OUT };
      fade(incoming, [{ opacity: 0, transform: `translateX(${SLIDE * dir}px)` }, { opacity: 1, transform: 'none' }], inOptions);
      if (relayout) controls.forEach((el) => fade(el, [{ opacity: 0 }, { opacity: 1 }], inOptions));
    })
    .catch(() => {}); // cancelled by a quicker click
}

const changing = () => performance.now() < changingUntil;

$('#back').addEventListener('click', () => !changing() && goStep(step - 1));

$('#next').addEventListener('click', async () => {
  if (changing()) return;
  if (step < NEXT_LABELS.length - 1) return goStep(step + 1);
  if (await save({ showWelcome: false })) show('colors');
});

// "You're all set" lists the same keys, read-only.
onRender((state) => {
  for (const el of $$('[data-caps]')) el.innerHTML = keycaps(bindingKeys(state.settings[el.dataset.caps]));
  for (const el of $$('[data-shortcut]')) el.innerHTML = keycaps(shortcutKeys(state.settings.shortcut));
});
