import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import { MAX_HOSTS_BYTES, assertHostsSize, buildHosts } from '../scripts/dnssink/build-hosts.mjs'

beforeEach(() => {
  faker.seed(20261110)
})

test('domínio de anúncio da lista sai como 0.0.0.0', () => {
  const domain = faker.internet.domainName()
  const raw = [
    '# StevenBlack comment',
    '127.0.0.1 localhost',
    '::1 localhost',
    `0.0.0.0 ${domain}`,
    `127.0.0.1 ads.${domain}`,
    '',
  ].join('\n')

  const out = buildHosts(raw, '')
  assert.match(out, new RegExp(`^0\\.0\\.0\\.0 ${domain}$`, 'm'))
  assert.match(out, new RegExp(`^0\\.0\\.0\\.0 ads\\.${domain}$`, 'm'))
  assert.doesNotMatch(out, /localhost/)
  assert.doesNotMatch(out, /^#/m)
})

test('domínio na whitelist não entra no hosts de saída', () => {
  const blocked = faker.internet.domainName()
  const allowed = faker.internet.domainName()
  const raw = `0.0.0.0 ${blocked}\n0.0.0.0 ${allowed}\n`
  const out = buildHosts(raw, `# keep\n${allowed}\n`)
  assert.match(out, new RegExp(`^0\\.0\\.0\\.0 ${blocked}$`, 'm'))
  assert.doesNotMatch(out, new RegExp(`0\\.0\\.0\\.0 ${allowed}`))
})

test('arquivo acima do teto cita bytes recebidos e o teto', () => {
  assert.equal(MAX_HOSTS_BYTES, 20 * 1024 * 1024)
  const n = faker.number.int({ min: 80, max: 200 })
  const cap = faker.number.int({ min: 10, max: 40 })
  const text = 'x'.repeat(n)
  assert.throws(
    () => assertHostsSize(text, cap),
    (err) => {
      assert.match(String(err.message), new RegExp(String(n)))
      assert.match(String(err.message), new RegExp(String(cap)))
      return true
    },
  )
  assert.doesNotThrow(() => assertHostsSize(text, n))
})
