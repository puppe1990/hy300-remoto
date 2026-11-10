/** Dual of src-tauri/src/host.rs parse_host */

export const DEFAULT_HOST = '192.168.15.4:3268'

export function parseHost(raw) {
  const trimmed = String(raw ?? '').trim()
  if (!trimmed) {
    return {
      ok: false,
      error: `host vazio ${JSON.stringify(raw)}, esperado host:porta (ex. ${DEFAULT_HOST})`,
    }
  }
  const colon = trimmed.lastIndexOf(':')
  if (colon <= 0) {
    return {
      ok: false,
      error: `host ${JSON.stringify(trimmed)} sem porta, esperado host:porta (ex. ${DEFAULT_HOST})`,
    }
  }
  const name = trimmed.slice(0, colon)
  const portS = trimmed.slice(colon + 1)
  const port = Number(portS)
  if (!Number.isInteger(port)) {
    return {
      ok: false,
      error: `porta ${JSON.stringify(portS)} em ${JSON.stringify(trimmed)} não é número, esperado 1–65535`,
    }
  }
  if (port < 1 || port > 65535) {
    return {
      ok: false,
      error: `porta ${port} em ${JSON.stringify(trimmed)} fora de 1–65535`,
    }
  }
  if (!/^[A-Za-z0-9._-]+$/.test(name)) {
    return {
      ok: false,
      error: `host ${JSON.stringify(trimmed)} tem caractere inválido, esperado hostname ou IPv4`,
    }
  }
  return { ok: true, host: `${name}:${port}` }
}
