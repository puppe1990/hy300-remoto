/** Dual of src-tauri/src/keys.rs — keep goldens in both suites. */

export const KEYCODES = {
  up: 19,
  down: 20,
  left: 21,
  right: 22,
  ok: 23,
  back: 4,
  home: 3,
  vol_up: 24,
  vol_down: 25,
  power: 26,
}

const EVENT_TO_PAD = {
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
  Enter: 'ok',
  NumpadEnter: 'ok',
  Escape: 'back',
  KeyH: 'home',
  Equal: 'vol_up',
  NumpadAdd: 'vol_up',
  Minus: 'vol_down',
  NumpadSubtract: 'vol_down',
}

export function fromPad(id) {
  return Object.hasOwn(KEYCODES, id) ? id : null
}

export function keycode(id) {
  const pad = fromPad(id)
  return pad ? KEYCODES[pad] : null
}

export function fromEventCode(code) {
  return EVENT_TO_PAD[code] ?? null
}
