import assert from 'node:assert/strict'
import { test } from 'node:test'
import { faker } from '@faker-js/faker'
import { DEFAULT_HOST, parseHost } from '../web/js/host.mjs'

test('padrão é o HY300 desta LAN', () => {
  assert.equal(DEFAULT_HOST, '192.168.15.4:3268')
})

test('aceita IPv4 e hostname com porta', () => {
  const port = faker.number.int({ min: 1024, max: 40000 })
  const parsed = parseHost(`  10.0.0.9:${port} `)
  assert.deepEqual(parsed, { ok: true, host: `10.0.0.9:${port}` })
  assert.deepEqual(parseHost('BZ-CCF3059164E0.local:3268'), {
    ok: true,
    host: 'BZ-CCF3059164E0.local:3268',
  })
})

test('rejeita vazio e porta fora da faixa citando o valor', () => {
  const empty = parseHost('')
  assert.equal(empty.ok, false)
  assert.match(empty.error, /vazio/)
  assert.match(empty.error, /host:porta/)

  const noPort = parseHost('sem-porta')
  assert.equal(noPort.ok, false)
  assert.match(noPort.error, /sem-porta/)

  const zero = parseHost('192.168.15.4:0')
  assert.equal(zero.ok, false)
  assert.match(zero.error, /1–65535/)

  const huge = parseHost('192.168.15.4:99999')
  assert.equal(huge.ok, false)
  assert.match(huge.error, /99999/)
})
