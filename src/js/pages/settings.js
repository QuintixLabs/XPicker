// ============================================================
// pages/settings.js
// The Settings page: switches, keys and resets.
// ============================================================

import { $, $$, ACTIONS, DEFAULTS, app, invoke, onRender, save } from '../core/state.js';
import { confirmAction } from '../window/dialog.js';

// ------------------------------------------------------------
// Switches and choices
// ------------------------------------------------------------

// data-options is "value:label,…", or "value:icon:name,…" with data-kind="icons".
function renderChoices(state) {
  for (const el of $$('.segmented[data-field]')) {
    const current = String(state.settings[el.dataset.field]);
    const icons = el.dataset.kind === 'icons';
    el.innerHTML = el.dataset.options
      .split(',')
      .map((option) => {
        const [value, label, name] = option.split(':');
        const content = icons ? `<i class="icon icon-${label}"></i>` : label;
        const title = icons ? ` title="${name}" aria-label="${name}"` : '';
        return `<button class="${value === current ? 'is-active' : ''}" data-value="${value}"${title}>${content}</button>`;
      })
      .join('');
  }

  for (const el of $$('.toggle')) {
    el.setAttribute('aria-checked', String(Boolean(state.settings[el.dataset.field])));
  }
}

// macOS keeps tray icons in the menu bar.
if (navigator.userAgent.includes('Mac')) {
  for (const el of $$('[data-tray-hint]')) el.textContent = 'Show XPicker in the menu bar';
}

onRender(renderChoices);

for (const el of $$('.segmented[data-field]')) {
  el.addEventListener('click', (e) => {
    const button = e.target.closest('[data-value]');
    if (!button) return;
    const raw = button.dataset.value;
    save({ [el.dataset.field]: Number.isNaN(Number(raw)) ? raw : Number(raw) });
  });
}

for (const el of $$('.toggle')) {
  el.addEventListener('click', () => save({ [el.dataset.field]: !app.state.settings[el.dataset.field] }));
}

// ------------------------------------------------------------
// Reset
// ------------------------------------------------------------

const SHORTCUT_FIELDS = ['shortcut', ...Object.keys(ACTIONS)];

$('#reset-shortcuts').addEventListener('click', async () => {
  const ok = await confirmAction({
    title: 'Reset shortcuts?',
    text: 'Your keys go back to the defaults. Other settings stay as they are.',
    action: 'Reset',
  });
  if (ok) save(Object.fromEntries(SHORTCUT_FIELDS.map((f) => [f, DEFAULTS[f]])));
});

$('#reset').addEventListener('click', async () => {
  const ok = await confirmAction({
    title: 'Reset all settings?',
    text: 'Everything goes back to its default. Your colors are kept.',
    action: 'Reset',
  });
  if (ok) save(DEFAULTS);
});

// ------------------------------------------------------------
// Folder and links
// ------------------------------------------------------------

$('#open-folder').addEventListener('click', () => invoke('open_settings_folder'));

for (const button of $$('[data-link]')) {
  button.addEventListener('click', () => invoke('open_link', { name: button.dataset.link }));
}
