// ============================================================
// core/state.js
// The settings and colors the main window shows, and saving them.
// Every page and component reads from here.
// ============================================================

export const { invoke } = window.__TAURI__.core;
export const { listen } = window.__TAURI__.event;

export const $ = (selector) => document.querySelector(selector);
export const $$ = (selector) => [...document.querySelectorAll(selector)];

// The keys used inside the picker, and what they're called.
export const ACTIONS = { pick: 'Copy color', cycle: 'Switch format', cancel: 'Cancel', open: 'Open XPicker' };

// What "Reset all settings" goes back to. Matches Settings::default in settings/mod.rs.
export const DEFAULTS = {
  shortcut: 'Ctrl+Shift+KeyX',
  pick: 'Mouse:2',
  cycle: 'Key:Control',
  cancel: 'Key:Escape',
  open: 'Key:Tab',
  theme: 'system',
  notify: true,
  historySize: 10,
  startWithComputer: false,
  pickOnLaunch: false,
  closeToTray: true,
  autoUpdate: true,
};

/** Everything from the settings file: { settings, history, … }. */
export const app = { state: null };

// ------------------------------------------------------------
// Redrawing
// ------------------------------------------------------------

const renderers = [];

/** Pages and components register how to redraw themselves when anything changes. */
export function onRender(fn) {
  renderers.push(fn);
}

export function render() {
  for (const fn of renderers) fn(app.state);
}

// ------------------------------------------------------------
// Loading and saving
// ------------------------------------------------------------

export async function load() {
  app.state = await invoke('state');
  listen('state', (e) => {
    app.state = e.payload;
    render();
  });
  render();
}

/** Saves a change. Shows the error in the status line if it can't. */
export async function save(patch) {
  try {
    app.state = await invoke('save_settings', { settings: { ...app.state.settings, ...patch } });
    status();
    render();
    return true;
  } catch (err) {
    status(String(err), true);
    return false;
  }
}

/** The small status line. The welcome screen only ever shows errors there. */
export function status(text, error = false) {
  for (const el of $$('.status')) {
    const quiet = el.closest('.welcome') && !error;
    el.textContent = quiet ? '' : (text ?? '');
    el.classList.toggle('is-error', error);
  }
}
