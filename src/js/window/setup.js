// ============================================================
// window/setup.js
// GNOME's one-time setup (a small extension, so picking doesn't ask
// each time or play the camera sound): a note with a Set up button on
// the welcome screen and the Colors page, like Blip's.
// ============================================================

import { $$, invoke } from '../core/state.js';
import { reveal } from './reveal.js';

// Short enough for one line next to the button; "Set up" never splits.
const TEXT = 'XPicker needs a one-time setup on GNOME. Click Set up.';
const DONE = 'All set. Log out and back in once, and XPicker is ready.';

function fill(note) {
  if (note.querySelector('[data-allow]')) return;
  note.classList.add('notice-action');
  note.innerHTML = `
    <span class="notice-text"></span>
    <button class="btn btn-accent btn-sm allow-button" data-allow>
      <span class="when-ready">Set up</span><span class="when-waiting">Waiting…</span>
    </button>
    <div class="reveal notice-error"><div class="reveal-inner"><span class="notice-error-text"></span></div></div>`;
  note.querySelector('.notice-text').textContent = TEXT;
}

/** Shows the note until GNOME runs the extension; the welcome screen waits for
 *  it. `message` goes under it (what to do next, or what went wrong). */
function show(needed, message = '') {
  document.documentElement.classList.toggle('needs-setup', needed);
  for (const el of $$('.welcome .step-actions')) el.inert = needed;
  for (const wrap of $$('[data-setup]')) {
    const note = wrap.querySelector('.notice');
    if (needed) {
      fill(note);
      note.querySelector('.notice-error-text').textContent = message;
      reveal(note.querySelector('.notice-error'), Boolean(message));
    }
    reveal(wrap, needed);
  }
}

// "" once GNOME runs the extension (or off GNOME); otherwise it still needs it.
show(Boolean(await invoke('desktop_setup').catch(() => '')));

document.addEventListener('click', async (e) => {
  if (!e.target.closest('[data-allow]')) return;
  const buttons = $$('[data-allow]');
  for (const b of buttons) {
    b.disabled = true;
    b.classList.add('is-waiting');
  }
  let message = '';
  try {
    if ((await invoke('desktop_set_up')) === 'login') message = DONE;
  } catch (err) {
    message = String(err);
  }
  show(Boolean(await invoke('desktop_setup').catch(() => '')), message);
  for (const b of buttons) {
    b.disabled = false;
    b.classList.remove('is-waiting');
  }
});
