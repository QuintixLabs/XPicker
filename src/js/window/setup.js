// ============================================================
// window/setup.js
// GNOME's one-time setup (a small extension, so picking doesn't ask
// each time or play the camera sound): a note with a Set up button on
// the welcome screen and the Colors page.
// ============================================================

import { $$, invoke } from '../core/state.js';
import { reveal } from './reveal.js';

const TEXT = 'XPicker needs a quick one-time setup on GNOME, so it can pick colors without asking every time. Click Set up.';
const DONE = 'All set. Log out and back in once to finish.';

function fill(note) {
  if (note.querySelector('[data-allow]')) return;
  note.classList.add('notice-action');
  note.innerHTML = `
    <span class="notice-text"></span>
    <button class="btn btn-accent btn-sm allow-button" data-allow>
      <span class="when-ready">Set up</span><span class="when-waiting">Waiting…</span>
    </button>
    <div class="reveal notice-error"><div class="reveal-inner"><span class="notice-error-text"></span></div></div>`;
}

/** state: "" (nothing to do), "needed" or "login". Until it's set up, the
 *  welcome screen waits for it. */
function show(state, error = '') {
  document.documentElement.classList.toggle('needs-setup', state === 'needed');
  for (const el of $$('.welcome .step-actions')) el.inert = state === 'needed';
  for (const wrap of $$('[data-setup]')) {
    const note = wrap.querySelector('.notice');
    if (state) {
      fill(note);
      note.querySelector('.notice-text').textContent = state === 'login' ? DONE : TEXT;
      note.querySelector('[data-allow]').hidden = state === 'login';
      note.querySelector('.notice-error-text').textContent = error;
      reveal(note.querySelector('.notice-error'), Boolean(error));
    }
    reveal(wrap, Boolean(state));
  }
}

show(await invoke('desktop_setup').catch(() => ''));

document.addEventListener('click', async (e) => {
  if (!e.target.closest('[data-allow]')) return;
  const buttons = $$('[data-allow]');
  for (const b of buttons) {
    b.disabled = true;
    b.classList.add('is-waiting');
  }
  try {
    show(await invoke('desktop_set_up'));
  } catch (err) {
    show('needed', String(err));
  }
  for (const b of buttons) {
    b.disabled = false;
    b.classList.remove('is-waiting');
  }
});