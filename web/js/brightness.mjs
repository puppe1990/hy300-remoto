/** Dual of `src-tauri/src/brightness.rs` — keep goldens in both suites. */
export const BRIGHTNESS_PROP = 'persist.vendor.brightness.main'
export const BRIGHTNESS_MIN = 0
export const BRIGHTNESS_MAX = 100
export const BRIGHTNESS_STEP = 5
export const BRIGHTNESS_DEFAULT = 50

/**
 * Clamp picture brightness to the TXCZ 0–100 range.
 * @param {number} n
 * @returns {number}
 * @example
 * clampBrightness(140) // 100
 */
export function clampBrightness(n) {
  if (n < BRIGHTNESS_MIN) return BRIGHTNESS_MIN
  if (n > BRIGHTNESS_MAX) return BRIGHTNESS_MAX
  return n
}

/**
 * Parse `getprop persist.vendor.brightness.main`. Empty → 50.
 * @param {string} raw
 * @returns {{ok: true, value: number}|{ok: false, error: string}}
 * @example
 * parseBrightness('50\n') // { ok: true, value: 50 }
 */
export function parseBrightness(raw) {
  const text = String(raw ?? '').trim()
  if (!text) return { ok: true, value: BRIGHTNESS_DEFAULT }
  if (!/^-?\d+$/.test(text)) {
    return {
      ok: false,
      error: `brilho ${JSON.stringify(raw)}; expected inteiro 0–100`,
    }
  }
  const n = Number(text)
  if (n < BRIGHTNESS_MIN || n > BRIGHTNESS_MAX) {
    return {
      ok: false,
      error: `brilho ${JSON.stringify(raw)}; expected inteiro 0–100`,
    }
  }
  return { ok: true, value: n }
}

/**
 * Next brightness after a pad step. Saturates at 0 and 100.
 * @param {number} current
 * @param {number} delta
 * @returns {number}
 * @example
 * nudgeBrightness(47, 5) // 52
 */
export function nudgeBrightness(current, delta) {
  return clampBrightness(Number(current) + Number(delta))
}

/**
 * Pad id from a `[data-bright]` value, or null if unknown.
 * @param {string} id
 * @returns {string|null}
 * @example
 * fromPad('bright_up') // "bright_up"
 */
export function fromPad(id) {
  if (id === 'bright_down' || id === 'bright_up') return id
  return null
}

/**
 * Pad id from a KeyboardEvent.code, or null if unbound.
 * @param {string} code
 * @returns {string|null}
 * @example
 * fromEventCode('Comma') // "bright_down"
 */
export function fromEventCode(code) {
  if (code === 'Comma') return 'bright_down'
  if (code === 'Period') return 'bright_up'
  return null
}

/**
 * ±5 for a brightness pad id, or null if unknown.
 * @param {string} id
 * @returns {number|null}
 * @example
 * deltaFromPad('bright_down') // -5
 */
export function deltaFromPad(id) {
  const pad = fromPad(id)
  if (pad === 'bright_down') return -BRIGHTNESS_STEP
  if (pad === 'bright_up') return BRIGHTNESS_STEP
  return null
}
