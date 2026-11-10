import { fromEventCode, keycode } from './keys.mjs'
import { mapTap } from './tap.mjs'
import { parseHost } from './host.mjs'
import { PREVIEW_INTERVAL_MS, releasePreviewLock, statusCopy, takePreviewLock } from './preview.mjs'

const invoke = window.__TAURI__?.core?.invoke

const statusEl = document.getElementById('status')
const previewEl = document.getElementById('preview')
const wrapEl = document.getElementById('preview-wrap')
const hostInput = document.getElementById('host')
const hostForm = document.getElementById('host-form')

let screen = { width: 1280, height: 720 }
const previewLock = { inFlight: false }

function setStatus(kind, detail) {
  statusEl.textContent = statusCopy(kind, detail)
  statusEl.classList.toggle('is-error', kind === 'error')
  statusEl.classList.toggle('is-off', kind === 'disconnected')
}

async function sendKey(id) {
  const code = keycode(id)
  if (code == null || !invoke) return
  try {
    await invoke('send_key', { code })
    setStatus('connected')
  } catch (err) {
    setStatus('error', String(err))
  }
}

function onPreviewClick(event) {
  const rect = wrapEl.getBoundingClientRect()
  const hit = mapTap(event.clientX - rect.left, event.clientY - rect.top, rect, screen)
  if (!hit || !invoke) return
  invoke('send_tap', { x: hit.x, y: hit.y }).then(
    () => setStatus('connected'),
    (err) => setStatus('error', String(err)),
  )
}

function onKeyDown(event) {
  if (event.target === hostInput) return
  const pad = fromEventCode(event.code)
  if (!pad) return
  event.preventDefault()
  sendKey(pad)
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
    setStatus('connected')
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
    setStatus('connected')
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
    setStatus('connected')
  } catch (err) {
    setStatus('error', String(err))
  }
})

document.querySelectorAll('[data-key]').forEach((btn) => {
  btn.addEventListener('click', () => sendKey(btn.dataset.key))
})
wrapEl.addEventListener('click', onPreviewClick)
window.addEventListener('keydown', onKeyDown)
boot()
