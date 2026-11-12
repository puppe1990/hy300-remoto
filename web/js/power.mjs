/** Dual of `src-tauri/src/power.rs` — keep goldens in both suites. */
/** Android 12 window title for the power-off confirm (`LegacyGlobalActions`). */
export const POWER_OFF_FOCUS = 'GlobalActions'

/**
 * Lamp/user power from `dumpsys window displays`. ADB can still be up.
 *
 * @param {string} raw
 * @returns {'on' | 'off'}
 * @example
 * powerFromWindowDump('mCurrentFocus=Window{c85f1b1 u0 LegacyGlobalActions}') // "off"
 */
export function powerFromWindowDump(raw) {
  const text = String(raw ?? '')
  const line = text.split('\n').find((row) => row.includes('mCurrentFocus='))
  if (!line) return 'on'
  if (line.includes(POWER_OFF_FOCUS)) return 'off'
  return 'on'
}

/**
 * Header power: Android confirm or GPIO lamp rails.
 *
 * @param {'on'|'off'} focus
 * @param {boolean} lampOn
 * @returns {'on'|'off'}
 * @example
 * combinePower('on', false) // "off"
 */
export function combinePower(focus, lampOn) {
  if (focus === 'off' || lampOn === false) return 'off'
  return 'on'
}
