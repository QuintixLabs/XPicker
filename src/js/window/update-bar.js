// ============================================================
// window/update-bar.js
// Updates in the window: the "Check for updates" button and the
// "Restart to update now?" bar.
// ============================================================

import { $, invoke, listen } from '../core/state.js';

const button = $('#update');
const bar = $('#update-bar');
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

let appVersion = '';
let readyVersion = null;       // a downloaded update waiting to install
const dismissed = new Set();   // versions you said "Later" to

// ------------------------------------------------------------
// Version line
// ------------------------------------------------------------

window.__TAURI__.app.getVersion().then((v) => {
  appVersion = v;
  $('#version').textContent = `Version ${v}`;
});

// Automatic updates say what they're doing ("Updating to 1.0.4…"); empty when done.
listen('update-status', (e) => {
  $('#version').textContent = `Version ${appVersion}${e.payload ? ` · ${e.payload}` : ''}`;
});

// ------------------------------------------------------------
// The button
// ------------------------------------------------------------

const TEXT = {
  idle: 'Check for updates',
  checking: 'Checking…',
  latest: "You're up to date",
  updating: 'Updating…',
  ready: 'Restart to update',
  error: "Couldn't check for updates",
};

/** Cross-fades the button to a new state (the CSS shows the spinner or check for it). */
async function setState(state, text = TEXT[state]) {
  const content = button.querySelector('.update-content');
  content.classList.add('is-swapping');
  await wait(140);
  button.dataset.state = state;
  button.querySelector('.update-text').textContent = text;
  content.classList.remove('is-swapping');
}

/** Installs the waiting update; XPicker restarts and reopens on the new version. */
async function restartToUpdate() {
  bar.classList.remove('is-open');
  await setState('updating', 'Restarting…');
  try {
    await invoke('install_update');
  } catch {
    await setState('error', "Couldn't install the update");
    await wait(4000);
    setState(readyVersion ? 'ready' : 'idle');
  }
}

async function checkForUpdates() {
  const current = button.dataset.state;
  if (current === 'ready') return restartToUpdate();
  if (current !== 'idle') return;

  await setState('checking');
  const started = performance.now();
  try {
    const version = await invoke('check_update');
    await wait(Math.max(0, 800 - (performance.now() - started))); // let the spinner read as a check
    if (!version) {
      await setState('latest');
      await wait(3500);
      return setState('idle');
    }
    await setState('updating', `Updating to v${version}…`);
    await invoke('install_update'); // restarts into the new version when done
  } catch {
    await setState('error');
    await wait(4000);
    setState('idle');
  }
}

button.addEventListener('click', checkForUpdates);
listen('update-progress', (e) => {
  button.querySelector('.update-text').textContent = `Updating… ${e.payload}%`;
});

// ------------------------------------------------------------
// The bar: "XPicker 1.0.4 is ready. Restart to update now?"
// Only while an update is downloaded and this window is open. "Later" hides it
// for that version; it then installs by itself once the window is closed.
// ------------------------------------------------------------

function renderBar() {
  if (readyVersion) $('#update-bar-text').textContent = `XPicker ${readyVersion} is ready. Restart to update now?`;
  bar.classList.toggle('is-open', Boolean(readyVersion) && !dismissed.has(readyVersion));
}

function updateReady(version) {
  if (!version) return;
  readyVersion = version;
  renderBar();
  if (button.dataset.state === 'idle') setState('ready');
}

listen('update-ready', (e) => updateReady(e.payload));
invoke('pending_update').then(updateReady);

$('#update-restart').addEventListener('click', restartToUpdate);
$('#update-later').addEventListener('click', () => {
  dismissed.add(readyVersion);
  renderBar();
});
