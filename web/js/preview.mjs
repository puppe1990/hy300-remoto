export const PREVIEW_INTERVAL_MS = 500

export function takePreviewLock(state) {
  if (state.inFlight) return false
  state.inFlight = true
  return true
}

export function releasePreviewLock(state) {
  state.inFlight = false
}

export function statusCopy(kind, detail) {
  if (kind === 'connecting') return 'conectando…'
  if (kind === 'connected') return 'ligado'
  if (kind === 'disconnected') return 'sem conexão'
  if (kind === 'error') return `falhou: ${detail || 'erro desconhecido'}`
  return `falhou: status ${JSON.stringify(kind)} não é connecting|connected|disconnected|error`
}
