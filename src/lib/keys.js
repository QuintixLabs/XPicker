// Key + mouse bindings — how they're stored, matched and displayed.
// Bindings: "Mouse:<button>" or "Key:<code>". Global shortcut: "Ctrl+Shift+KeyX".

const IS_MAC = navigator.userAgent.includes('Mac');

const MODIFIERS = {
  ControlLeft: 'Control', ControlRight: 'Control',
  ShiftLeft: 'Shift', ShiftRight: 'Shift',
  AltLeft: 'Alt', AltRight: 'Alt',
  MetaLeft: 'Meta', MetaRight: 'Meta',
};

const NAMES = {
  'Mouse:0': 'Left click',
  'Mouse:1': 'Middle click',
  'Mouse:2': 'Right click',
  Control: 'Ctrl',
  Ctrl: 'Ctrl',
  Alt: IS_MAC ? 'Option' : 'Alt',
  Meta: IS_MAC ? 'Cmd' : 'Win',
  Super: IS_MAC ? 'Cmd' : 'Win',
  Escape: 'Esc',
  ArrowUp: '↑',
  ArrowDown: '↓',
  ArrowLeft: '←',
  ArrowRight: '→',
};

/** Physical key code; falls back to e.key for input without a scan code (remappers, remote desktop). */
function codeOf(e) {
  if (e.code) return MODIFIERS[e.code] ?? e.code;
  const key = e.key;
  if (key === ' ') return 'Space';
  if (/^[a-z]$/i.test(key)) return `Key${key.toUpperCase()}`;
  if (/^\d$/.test(key)) return `Digit${key}`;
  return key;
}

const isModifier = (code) => ['Control', 'Shift', 'Alt', 'Meta'].includes(code);

export const fromKey = (e) => `Key:${codeOf(e)}`;
export const fromMouse = (e) => `Mouse:${e.button}`;

function keyName(code) {
  if (NAMES[code]) return NAMES[code];
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  if (code.startsWith('Numpad')) return `Num ${code.slice(6)}`;
  return code;
}

/** "Mouse:2" → ["Right click"], "Key:Control" → ["Ctrl"] */
export function bindingKeys(binding) {
  if (binding.startsWith('Mouse:')) return [NAMES[binding] ?? `Mouse ${Number(binding.slice(6)) + 1}`];
  return [keyName(binding.slice(4))];
}

/** "Ctrl+Shift+KeyX" → ["Ctrl", "Shift", "X"] */
export function shortcutKeys(shortcut) {
  return shortcut ? shortcut.split('+').map(keyName) : [];
}

/** Global shortcut from a keydown; needs at least one modifier plus a regular key. */
export function shortcutFromKey(e) {
  const mods = [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift', e.metaKey && 'Super'].filter(Boolean);
  const code = codeOf(e);
  const complete = !isModifier(code) && mods.length > 0;
  return { mods, shortcut: complete ? [...mods, code].join('+') : null };
}
