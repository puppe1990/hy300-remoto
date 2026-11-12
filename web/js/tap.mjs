/** Dual of `src-tauri/src/tap.rs`. Native HY300 size. */
export const HY300_SCREEN = { width: 1280, height: 720 }

/**
 * Map a click in the preview box (`object-fit: contain`) to a projector pixel.
 * Letterbox clicks return null.
 * @param {number} clickX
 * @param {number} clickY
 * @param {{width: number, height: number}} preview
 * @param {{width: number, height: number}} [screen]
 * @returns {{x: number, y: number}|null}
 * @example
 * mapTap(640, 360, { width: 1280, height: 720 }) // { x: 640, y: 360 }
 */
export function mapTap(clickX, clickY, preview, screen = HY300_SCREEN) {
  if (preview.width <= 0 || preview.height <= 0 || screen.width === 0 || screen.height === 0) {
    return null
  }
  const scale = Math.min(preview.width / screen.width, preview.height / screen.height)
  if (scale <= 0) {
    return null
  }
  const contentW = screen.width * scale
  const contentH = screen.height * scale
  const offX = (preview.width - contentW) / 2
  const offY = (preview.height - contentH) / 2
  if (clickX < offX || clickY < offY || clickX > offX + contentW || clickY > offY + contentH) {
    return null
  }
  const x = Math.round((clickX - offX) / scale)
  const y = Math.round((clickY - offY) / scale)
  return {
    x: Math.min(screen.width - 1, Math.max(0, x)),
    y: Math.min(screen.height - 1, Math.max(0, y)),
  }
}

/**
 * Map against the painted <img> once it has size. Wrap is fallback.
 * Reloading `src` each frame cancels `click`; use pointerdown.
 *
 * @param {{width: number, height: number}} wrap
 * @param {{width: number, height: number}|null|undefined} img
 * @returns {{width: number, height: number}}
 * @example
 * tapTargetBox({ width: 400, height: 300 }, { width: 400, height: 225 })
 */
export function tapTargetBox(wrap, img) {
  if (img && img.width > 0 && img.height > 0) return img
  return wrap
}

/**
 * Primary pointer only (left click / finger).
 * @param {number} button
 * @returns {boolean}
 * @example
 * previewPointerIsPrimary(0) // true
 */
export function previewPointerIsPrimary(button) {
  return button === 0
}
