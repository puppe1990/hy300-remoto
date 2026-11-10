/** Dual of src-tauri/src/tap.rs */

export const HY300_SCREEN = { width: 1280, height: 720 }

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
