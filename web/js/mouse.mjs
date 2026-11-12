/** Dual of `src-tauri/src/mouse.rs` — keep goldens in both suites. */
export const MOUSE_COALESCE_MS = 50

/**
 * Android `input mouse motionevent` action. Case-sensitive.
 * @param {string} value
 * @returns {'MOVE'|'DOWN'|'UP'|null}
 * @example
 * parseMouseAction('MOVE') // "MOVE"
 */
export function parseMouseAction(value) {
  if (value === 'MOVE' || value === 'DOWN' || value === 'UP') return value
  return null
}

/**
 * Error when the action is not MOVE|DOWN|UP.
 * @param {string} value
 * @returns {string}
 * @example
 * invalidMouseAction('tap') // includes tap and MOVE|DOWN|UP
 */
export function invalidMouseAction(value) {
  return `ação mouse ${value} não é MOVE|DOWN|UP`
}

/**
 * Skip MOVE when (x, y) matches the last sent mouse event.
 * @param {{action: string, x: number, y: number}|null} last
 * @param {{action: string, x: number, y: number}} next
 * @returns {boolean}
 * @example
 * shouldSkipDuplicateMove({ action: 'MOVE', x: 1, y: 1 }, { action: 'MOVE', x: 1, y: 1 }) // true
 */
export function shouldSkipDuplicateMove(last, next) {
  if (!next || next.action !== 'MOVE') return false
  if (!last) return false
  return last.x === next.x && last.y === next.y
}

/**
 * Flip the session mouse mode.
 * @param {boolean} on
 * @returns {boolean}
 * @example
 * toggleMouseMode(false) // true
 */
export function toggleMouseMode(on) {
  return !on
}

/**
 * Esc with mode on exits and must not send BACK.
 * @param {boolean} on
 * @param {string} eventCode
 * @returns {{on: boolean, consume: boolean}}
 * @example
 * escapeExitsMouseMode(true, 'Escape') // { on: false, consume: true }
 */
export function escapeExitsMouseMode(on, eventCode) {
  if (on && eventCode === 'Escape') return { on: false, consume: true }
  return { on, consume: false }
}
