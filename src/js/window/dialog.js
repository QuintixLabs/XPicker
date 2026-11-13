// ============================================================
// window/dialog.js
// The "Are you sure?" box, with its fade.
// ============================================================

import { $ } from '../core/state.js';

const dialog = $('#confirm');

/** Asks first. Resolves true if the main button was pressed. `danger`: a red button. */
export function confirmAction({ title, text, action, danger = true }) {
  $('#confirm-title').textContent = title;
  $('#confirm-text').textContent = text;
  $('#confirm-ok').textContent = action;
  $('#confirm-ok').className = danger ? 'btn btn-danger' : 'btn btn-accent';
  dialog.returnValue = '';
  dialog.classList.remove('is-closing');
  dialog.showModal();
  document.activeElement?.blur(); // no pre-focused button ring on open
  return new Promise((resolve) => {
    dialog.addEventListener('close', () => resolve(dialog.returnValue === 'ok'), { once: true });
  });
}

/** Plays the fade-out, then really closes. Buttons and Esc both come through here. */
function close(value) {
  if (dialog.classList.contains('is-closing')) return;
  dialog.classList.add('is-closing');
  dialog.addEventListener('animationend', () => dialog.close(value), { once: true });
}

dialog.querySelector('form').addEventListener('submit', (e) => {
  e.preventDefault();
  close(e.submitter?.value ?? '');
});

dialog.addEventListener('cancel', (e) => {
  e.preventDefault();
  close('');
});
