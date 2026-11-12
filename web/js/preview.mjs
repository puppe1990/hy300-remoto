/** Dual of `src-tauri/src/preview.rs`. 2 fps; one screencap at a time. */
export const PREVIEW_INTERVAL_MS = 500

/**
 * Acquire the in-flight lock. False if a frame is already running.
 * @param {{inFlight: boolean}} state
 * @returns {boolean}
 * @example
 * takePreviewLock({ inFlight: false }) // true
 */
export function takePreviewLock(state) {
  if (state.inFlight) return false
  state.inFlight = true
  return true
}

/**
 * Drop the in-flight lock so the next 500 ms tick can capture.
 * @param {{inFlight: boolean}} state
 * @returns {void}
 */
export function releasePreviewLock(state) {
  state.inFlight = false
}

/**
 * Portuguese status line. Unknown kinds include the received value.
 * @param {string} kind
 * @param {string} [detail]
 * @returns {string}
 * @example
 * statusCopy('connected') // "ligado"
 */
export function statusCopy(kind, detail) {
  if (kind === 'connecting') return 'conectando…'
  if (kind === 'connected') return 'ligado'
  if (kind === 'disconnected') return 'sem conexão'
  if (kind === 'off') return 'desligado'
  if (kind === 'error') return `falhou: ${detail || 'erro desconhecido'}`
  return `falhou: status ${JSON.stringify(kind)} não é connecting|connected|disconnected|off|error`
}
