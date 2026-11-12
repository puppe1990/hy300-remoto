/** Dual of `src-tauri/src/swipe.rs` — keep goldens in both suites. */
export const SWIPE_COALESCE_MS = 50
export const SWIPE_DURATION_MS = 200
export const SWIPE_MIN_TRAVEL = 120
export const SWIPE_MAX_TRAVEL = 240

/**
 * Convert a WheelEvent deltaY into pixels. Unknown deltaMode is ignored.
 * @param {number} deltaY
 * @param {number} deltaMode
 * @returns {number|null}
 * @example
 * normalizeWheelDelta(2, 1) // 32
 */
export function normalizeWheelDelta(deltaY, deltaMode) {
  if (deltaMode === 0) return Number(deltaY)
  if (deltaMode === 1) return Number(deltaY) * 16
  if (deltaMode === 2) return Number(deltaY) * 400
  return null
}

/**
 * Finger travel in pixels. One notch saturates at 120; a flick at 240.
 * @param {number} absNormalizedDelta
 * @returns {number}
 * @example
 * swipeTravel(4) // 120
 */
export function swipeTravel(absNormalizedDelta) {
  const raw = Math.round(Math.abs(Number(absNormalizedDelta)) * 4)
  if (raw < SWIPE_MIN_TRAVEL) return SWIPE_MIN_TRAVEL
  if (raw > SWIPE_MAX_TRAVEL) return SWIPE_MAX_TRAVEL
  return raw
}

/**
 * Map normalized wheel delta + origin pixel to a vertical `input swipe`.
 * `deltaY > 0` (scroll down) moves the finger up. Origin null → null.
 * @param {number} normalizedDeltaY
 * @param {{x: number, y: number}|null} origin
 * @param {{width: number, height: number}} screen
 * @returns {{x1: number, y1: number, x2: number, y2: number, durationMs: number}|null}
 * @example
 * wheelToSwipe(30, { x: 640, y: 360 }, { width: 1280, height: 720 }).y2 // 240
 */
export function wheelToSwipe(normalizedDeltaY, origin, screen) {
  if (origin == null) return null
  if (!Number.isFinite(normalizedDeltaY) || normalizedDeltaY === 0) return null
  if (!screen || screen.width === 0 || screen.height === 0) return null
  const travel = swipeTravel(Math.abs(normalizedDeltaY))
  const dir = normalizedDeltaY > 0 ? -1 : 1
  const x1 = origin.x
  const y1 = origin.y
  const maxY = screen.height - 1
  const y2 = Math.min(maxY, Math.max(0, y1 + dir * travel))
  if (y2 === y1) return null
  return { x1, y1, x2: x1, y2, durationMs: SWIPE_DURATION_MS }
}

/**
 * Acquire the in-flight swipe lock. False if a swipe is already running.
 * @param {{inFlight: boolean}} state
 * @returns {boolean}
 * @example
 * takeSwipeLock({ inFlight: false }) // true
 */
export function takeSwipeLock(state) {
  if (state.inFlight) return false
  state.inFlight = true
  return true
}

/**
 * Drop the in-flight swipe lock so a coalesced burst can fire.
 * @param {{inFlight: boolean}} state
 * @returns {void}
 */
export function releaseSwipeLock(state) {
  state.inFlight = false
}
