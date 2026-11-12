import { fromEventCode, keycode } from './keys.mjs'
import {
  BRIGHTNESS_DEFAULT,
  deltaFromPad,
  fromEventCode as fromBrightEventCode,
} from './brightness.mjs'
import { lampOnFromPad } from './lamp.mjs'
import { mapTap, previewPointerIsPrimary, tapTargetBox } from './tap.mjs'
import {
  MOUSE_COALESCE_MS,
  escapeExitsMouseMode,
  parseMouseAction,
  shouldSkipDuplicateMove,
  toggleMouseMode,
} from './mouse.mjs'
import { parseHost } from './host.mjs'
import { PREVIEW_INTERVAL_MS, releasePreviewLock, statusCopy, takePreviewLock } from './preview.mjs'
import {
  SWIPE_COALESCE_MS,
  normalizeWheelDelta,
  releaseSwipeLock,
  takeSwipeLock,
  wheelToSwipe,
} from './swipe.mjs'

const invoke = window.__TAURI__?.core?.invoke

const statusEl = document.getElementById('status')
const previewEl = document.getElementById('preview')
const wrapEl = document.getElementById('preview-wrap')
const hostInput = document.getElementById('host')
const hostForm = document.getElementById('host-form')
const brightValEl = document.getElementById('bright-val')
const mouseToggleEl = document.getElementById('mouse-toggle')

let screen = { width: 1280, height: 720 }
const previewLock = { inFlight: false }
const swipeLock = { inFlight: false }
const swipeBurst = { accum: 0, clientX: 0, clientY: 0, timer: 0 }
let mouseOn = false
const mouseLock = { inFlight: false }
const mouseBurst = { x: 0, y: 0, hasMove: false, timer: 0 }
let lastMouse = null
let mouseHeld = false
let pendingClick = null

function setStatus(kind, detail) {
  statusEl.textContent = statusCopy(kind, detail)
  statusEl.classList.toggle('is-error', kind === 'error')
  statusEl.classList.toggle('is-off', kind === 'disconnected' || kind === 'off')
}

async function sendKey(id) {
  const code = keycode(id)
  if (code == null || !invoke) return
  try {
    await invoke('send_key', { code })
  } catch (err) {
    setStatus('error', String(err))
  }
}

function previewRect() {
  return tapTargetBox(wrapEl.getBoundingClientRect(), previewEl.getBoundingClientRect())
}

function hitFromEvent(event) {
  const rect = previewRect()
  return mapTap(event.clientX - rect.left, event.clientY - rect.top, rect, screen)
}

function paintMouseMode() {
  mouseToggleEl.setAttribute('aria-pressed', mouseOn ? 'true' : 'false')
  mouseToggleEl.classList.toggle('is-on', mouseOn)
  wrapEl.classList.toggle('is-mouse', mouseOn)
  wrapEl.setAttribute('aria-label', mouseOn ? 'Tela do projetor — mouse' : 'Tela do projetor')
}

function setMouseMode(on) {
  if (!on && mouseHeld && lastMouse) {
    mouseHeld = false
    queueMouse('UP', lastMouse.x, lastMouse.y)
  }
  mouseOn = on
  if (!on) cancelMouseMoveTimer()
  paintMouseMode()
}

function queueMouse(action, x, y) {
  const parsed = parseMouseAction(action)
  if (!parsed || !invoke) return
  const next = { action: parsed, x, y }
  if (parsed === 'MOVE') {
    queueMouseMove(next)
    return
  }
  cancelMouseMoveTimer()
  if (parsed === 'DOWN') {
    mouseBurst.hasMove = false
    void sendMouseNow(next)
    return
  }
  flushPendingMoveThen(next)
}

function queueMouseMove(next) {
  if (shouldSkipDuplicateMove(lastMouse, next) && !mouseBurst.hasMove) return
  mouseBurst.x = next.x
  mouseBurst.y = next.y
  mouseBurst.hasMove = true
  if (!mouseBurst.timer) {
    mouseBurst.timer = window.setTimeout(flushMouseMove, MOUSE_COALESCE_MS)
  }
}

function cancelMouseMoveTimer() {
  if (!mouseBurst.timer) return
  window.clearTimeout(mouseBurst.timer)
  mouseBurst.timer = 0
}

function flushPendingMoveThen(up) {
  if (!mouseBurst.hasMove) {
    void sendMouseNow(up)
    return
  }
  const move = { action: 'MOVE', x: mouseBurst.x, y: mouseBurst.y }
  mouseBurst.hasMove = false
  if (shouldSkipDuplicateMove(lastMouse, move)) {
    void sendMouseNow(up)
    return
  }
  pendingClick = up
  void sendMouseNow(move)
}

function flushMouseMove() {
  mouseBurst.timer = 0
  if (!mouseBurst.hasMove) return
  const next = { action: 'MOVE', x: mouseBurst.x, y: mouseBurst.y }
  mouseBurst.hasMove = false
  if (shouldSkipDuplicateMove(lastMouse, next)) return
  void sendMouseNow(next)
}

async function sendMouseNow(next) {
  if (mouseLock.inFlight) {
    holdWhileMouseBusy(next)
    return
  }
  mouseLock.inFlight = true
  try {
    await invoke('send_mouse', next)
    lastMouse = next
  } catch (err) {
    setStatus('error', String(err))
  } finally {
    mouseLock.inFlight = false
    drainMouseQueue()
  }
}

function holdWhileMouseBusy(next) {
  if (next.action === 'MOVE') {
    mouseBurst.x = next.x
    mouseBurst.y = next.y
    mouseBurst.hasMove = true
    return
  }
  pendingClick = next
}

function drainMouseQueue() {
  if (pendingClick) {
    const click = pendingClick
    pendingClick = null
    void sendMouseNow(click)
    return
  }
  if (mouseBurst.hasMove) flushMouseMove()
}

function onPreviewPointerDown(event) {
  if (!previewPointerIsPrimary(event.button)) return
  const hit = hitFromEvent(event)
  if (!hit || !invoke) return
  if (mouseOn) {
    mouseHeld = true
    wrapEl.setPointerCapture(event.pointerId)
    queueMouse('DOWN', hit.x, hit.y)
    return
  }
  invoke('send_tap', { x: hit.x, y: hit.y }).catch((err) => setStatus('error', String(err)))
}

function onPreviewPointerMove(event) {
  if (!mouseOn) return
  const hit = hitFromEvent(event)
  if (!hit) return
  queueMouse('MOVE', hit.x, hit.y)
}

function releasePreviewCapture(event) {
  if (!wrapEl.hasPointerCapture(event.pointerId)) return
  wrapEl.releasePointerCapture(event.pointerId)
}

function onPreviewPointerUp(event) {
  if (!mouseOn || !mouseHeld) return
  mouseHeld = false
  releasePreviewCapture(event)
  const hit = hitFromEvent(event)
  const x = hit?.x ?? lastMouse?.x
  const y = hit?.y ?? lastMouse?.y
  if (x == null || y == null) return
  queueMouse('UP', x, y)
}

function onKeyDown(event) {
  if (event.target === hostInput) return
  const mouseEsc = escapeExitsMouseMode(mouseOn, event.code)
  if (mouseEsc.consume) {
    event.preventDefault()
    setMouseMode(mouseEsc.on)
    return
  }
  const bright = fromBrightEventCode(event.code)
  if (bright) {
    event.preventDefault()
    const delta = deltaFromPad(bright)
    if (delta != null) nudgeBrightness(delta)
    return
  }
  const pad = fromEventCode(event.code)
  if (!pad) return
  event.preventDefault()
  sendKey(pad)
}

function onPreviewWheel(event) {
  event.preventDefault()
  const norm = normalizeWheelDelta(event.deltaY, event.deltaMode)
  if (norm == null) return
  swipeBurst.clientX = event.clientX
  swipeBurst.clientY = event.clientY
  if (norm !== 0) swipeBurst.accum += norm
  if (!swipeBurst.timer) {
    swipeBurst.timer = window.setTimeout(flushSwipe, SWIPE_COALESCE_MS)
  }
}

function flushSwipe() {
  swipeBurst.timer = 0
  void sendAccumulatedSwipe()
}

function plannedFromBurst() {
  if (swipeBurst.accum === 0) return null
  const rect = previewRect()
  const origin = mapTap(swipeBurst.clientX - rect.left, swipeBurst.clientY - rect.top, rect, screen)
  if (!origin) {
    swipeBurst.accum = 0
    return null
  }
  const planned = wheelToSwipe(swipeBurst.accum, origin, screen)
  if (!planned) {
    swipeBurst.accum = 0
    return null
  }
  return planned
}

async function sendAccumulatedSwipe() {
  if (!invoke) return
  const planned = plannedFromBurst()
  if (!planned) return
  if (!takeSwipeLock(swipeLock)) return
  swipeBurst.accum = 0
  try {
    await invoke('send_swipe', planned)
  } catch (err) {
    setStatus('error', String(err))
  } finally {
    releaseSwipeLock(swipeLock)
    await sendAccumulatedSwipe()
  }
}

function showBrightness(value) {
  brightValEl.textContent = String(value)
}

async function nudgeBrightness(delta) {
  if (!invoke) return
  try {
    const value = await invoke('nudge_brightness', { delta })
    showBrightness(value)
  } catch (err) {
    setStatus('error', String(err))
  }
}

async function setLamp(id) {
  const on = lampOnFromPad(id)
  if (on == null || !invoke) return
  try {
    await invoke('set_lamp', { on })
    setStatus(on ? 'connected' : 'off')
  } catch (err) {
    setStatus('error', String(err))
  }
}

async function refreshPreview() {
  if (!invoke) return
  if (!takePreviewLock(previewLock)) return
  try {
    const frame = await invoke('preview_frame')
    if (frame?.dataUrl) {
      previewEl.src = frame.dataUrl
      previewEl.classList.add('is-on')
    }
    if (frame?.width && frame?.height) {
      screen = { width: frame.width, height: frame.height }
    }
    setStatus(frame?.power === 'off' ? 'off' : 'connected')
  } catch (err) {
    setStatus('error', String(err))
    try {
      await invoke('connect_host')
    } catch (reconnectErr) {
      setStatus('error', String(reconnectErr))
    }
  } finally {
    releasePreviewLock(previewLock)
  }
}

async function boot() {
  if (!invoke) {
    setStatus('error', 'Tauri ausente')
    return
  }
  try {
    const cfg = await invoke('get_host')
    hostInput.value = cfg.host
    setStatus('connecting')
    await invoke('connect_host')
    try {
      showBrightness(await invoke('get_brightness'))
    } catch (_err) {
      showBrightness(BRIGHTNESS_DEFAULT)
    }
  } catch (err) {
    setStatus('error', String(err))
  }
  await refreshPreview()
  setInterval(refreshPreview, PREVIEW_INTERVAL_MS)
}

hostForm.addEventListener('submit', async (event) => {
  event.preventDefault()
  const parsed = parseHost(hostInput.value)
  if (!parsed.ok) {
    setStatus('error', parsed.error)
    return
  }
  try {
    const cfg = await invoke('set_host', { host: parsed.host })
    hostInput.value = cfg.host
    setStatus('connecting')
    await invoke('connect_host')
    await refreshPreview()
  } catch (err) {
    setStatus('error', String(err))
  }
})

document.querySelectorAll('[data-key]').forEach((btn) => {
  btn.addEventListener('click', () => sendKey(btn.dataset.key))
})
document.querySelectorAll('[data-bright]').forEach((btn) => {
  btn.addEventListener('click', () => {
    const delta = deltaFromPad(btn.dataset.bright)
    if (delta != null) nudgeBrightness(delta)
  })
})
document.querySelectorAll('[data-lamp]').forEach((btn) => {
  btn.addEventListener('click', () => setLamp(btn.dataset.lamp))
})
mouseToggleEl.addEventListener('click', () => {
  setMouseMode(toggleMouseMode(mouseOn))
})
wrapEl.addEventListener('pointerdown', onPreviewPointerDown)
wrapEl.addEventListener('pointermove', onPreviewPointerMove)
wrapEl.addEventListener('pointerup', onPreviewPointerUp)
wrapEl.addEventListener('pointercancel', onPreviewPointerUp)
wrapEl.addEventListener('wheel', onPreviewWheel, { passive: false })
window.addEventListener('keydown', onKeyDown)
boot()
