/** Dual of `src-tauri/src/lamp.rs` — keep goldens in both suites. */
/** TXCZ LED rails. Not KEYCODE_POWER (that opens Android shutdown). */
export const LAMP_EN_PATH = '/sys/class/leds/Lcd_en/brightness'
export const LAMP_POWER_PATH = '/sys/class/leds/Lcd_power/brightness'
export const LAMP_ON_VALUE = 255
export const LAMP_OFF_VALUE = 0

/**
 * Parse `cat Lcd_power/brightness`. Empty or 0 → off.
 *
 * @param {string} raw
 * @returns {{ok: true, on: boolean} | {ok: false, error: string}}
 * @example
 * parseLamp('255\n') // { ok: true, on: true }
 */
export function parseLamp(raw) {
  const text = String(raw ?? '').trim()
  if (text === '') return { ok: true, on: false }
  if (!/^\d+$/.test(text)) {
    return {
      ok: false,
      error: `brilho da tela ${JSON.stringify(raw)} não é 0 ou 255`,
    }
  }
  return { ok: true, on: Number(text) > 0 }
}

/**
 * Pad id from a `[data-lamp]` value.
 * @param {string} id
 * @returns {'lamp_on'|'lamp_off'|null}
 * @example
 * fromPad('lamp_off') // "lamp_off"
 */
export function fromPad(id) {
  if (id === 'lamp_on') return 'lamp_on'
  if (id === 'lamp_off') return 'lamp_off'
  return null
}

/**
 * @param {string} id
 * @returns {boolean|null}
 */
export function lampOnFromPad(id) {
  if (id === 'lamp_on') return true
  if (id === 'lamp_off') return false
  return null
}
