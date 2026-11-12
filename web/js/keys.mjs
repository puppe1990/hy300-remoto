/**
 * Dual of `src-tauri/src/keys.rs` — keep goldens in both suites.
 * AOSP KeyEvent codes for pad ids.
 */
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
  mute: 164,
  power: 26,
  rew: 89,
  ff: 90,
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
  KeyM: 'mute',
  BracketLeft: 'rew',
  BracketRight: 'ff',
}

/**
 * Pad id from a `[data-key]` value, or null if unknown.
 * @param {string} id
 * @returns {string|null}
 * @example
 * fromPad('ok') // "ok"
 */
export function fromPad(id) {
  return Object.hasOwn(KEYCODES, id) ? id : null
}

/**
 * AOSP keycode for a pad id, or null if unknown.
 * @param {string} id
 * @returns {number|null}
 * @example
 * keycode('ok') // 23
 */
export function keycode(id) {
  const pad = fromPad(id)
  return pad ? KEYCODES[pad] : null
}

/**
 * Pad id from a KeyboardEvent.code, or null if unbound.
 * @param {string} code
 * @returns {string|null}
 * @example
 * fromEventCode('ArrowUp') // "up"
 */
export function fromEventCode(code) {
  return EVENT_TO_PAD[code] ?? null
}
