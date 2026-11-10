import { readFileSync } from 'node:fs'
import { pathToFileURL } from 'node:url'

export const MAX_HOSTS_BYTES = 20 * 1024 * 1024
export const STEVENBLACK_URL = 'https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts'

const SKIP_NAMES = new Set([
  'localhost',
  'localhost.localdomain',
  'local',
  'broadcasthost',
  'ip6-localhost',
  'ip6-loopback',
])

/**
 * Throw if hosts text exceeds the byte cap. Default cap is 20 MB (HY300 RAM).
 * @param {string} text
 * @param {number} [maxBytes]
 * @example
 * assertHostsSize('0.0.0.0 ads.test\n')
 */
export function assertHostsSize(text, maxBytes = MAX_HOSTS_BYTES) {
  const n = Buffer.byteLength(text)
  if (n > maxBytes) {
    throw new Error(`hosts ${n} bytes, teto ${maxBytes}`)
  }
}

/**
 * Parse whitelist: one domain per line, ignore comments and blanks.
 * @param {string} whitelistText
 * @returns {Set<string>}
 */
export function parseWhitelist(text) {
  const names = new Set()
  for (const line of String(text ?? '').split(/\r?\n/)) {
    const name = line.trim().split(/\s+/)[0] ?? ''
    if (!name || name.startsWith('#')) continue
    names.add(name.toLowerCase())
  }
  return names
}

/**
 * Keep StevenBlack host rows as `0.0.0.0 domain`, drop localhost and whitelist.
 * @param {string} raw
 * @param {string} [whitelistText]
 * @returns {string}
 * @example
 * buildHosts('0.0.0.0 ads.test\\n', 'ads.test') // ''
 */
export function buildHosts(raw, whitelistText = '') {
  const skip = parseWhitelist(whitelistText)
  const seen = new Set()
  const lines = []
  for (const line of String(raw ?? '').split(/\r?\n/)) {
    const domain = hostDomain(line)
    if (!domain || skip.has(domain) || seen.has(domain)) continue
    seen.add(domain)
    lines.push(`0.0.0.0 ${domain}`)
  }
  return lines.length ? `${lines.join('\n')}\n` : ''
}

function hostDomain(line) {
  const trimmed = line.trim()
  if (!trimmed || trimmed.startsWith('#')) return null
  const parts = trimmed.split(/\s+/)
  if (parts.length < 2) return null
  if (parts[0] !== '0.0.0.0' && parts[0] !== '127.0.0.1') return null
  const domain = parts[1].toLowerCase()
  if (SKIP_NAMES.has(domain)) return null
  return domain
}

function main() {
  const rawPath = process.argv[2]
  const whitePath = process.argv[3]
  if (!rawPath) {
    throw new Error(
      `argv ${JSON.stringify(process.argv.slice(2))} vazio, esperado raw.txt [whitelist.txt]`,
    )
  }
  const raw = readFileSync(rawPath, 'utf8')
  const whitelistText = whitePath ? readFileSync(whitePath, 'utf8') : ''
  const out = buildHosts(raw, whitelistText)
  assertHostsSize(out)
  process.stdout.write(out)
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main()
}
