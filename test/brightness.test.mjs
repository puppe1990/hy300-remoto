import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import {
  BRIGHTNESS_DEFAULT,
  BRIGHTNESS_MAX,
  BRIGHTNESS_MIN,
  BRIGHTNESS_PROP,
  BRIGHTNESS_STEP,
  clampBrightness,
  deltaFromPad,
  fromEventCode,
  fromPad,
  nudgeBrightness,
  parseBrightness,
} from '../web/js/brightness.mjs'

beforeEach(() => {
  faker.seed(20261111)
})

test('constantes do PQ TXCZ 0–100 passo 5', () => {
  assert.equal(BRIGHTNESS_PROP, 'persist.vendor.brightness.main')
  assert.equal(BRIGHTNESS_MIN, 0)
  assert.equal(BRIGHTNESS_MAX, 100)
  assert.equal(BRIGHTNESS_STEP, 5)
  assert.equal(BRIGHTNESS_DEFAULT, 50)
})

test('parse lê inteiro com newline; vazio vira o default 50', () => {
  const n = faker.number.int({ min: 0, max: 100 })
  assert.deepEqual(parseBrightness(`${n}\n`), { ok: true, value: n })
  assert.deepEqual(parseBrightness(''), { ok: true, value: BRIGHTNESS_DEFAULT })
  assert.deepEqual(parseBrightness('  '), { ok: true, value: BRIGHTNESS_DEFAULT })
})

test('parse recusa valor que não é inteiro 0–100 e cita o recebido', () => {
  const junk = faker.string.alpha({ length: { min: 4, max: 10 } })
  const err = parseBrightness(junk)
  assert.equal(err.ok, false)
  assert.match(err.error, new RegExp(junk))
  assert.match(err.error, /0–100/)
})

test('nudge satura em 0 e 100', () => {
  assert.equal(nudgeBrightness(0, -BRIGHTNESS_STEP), 0)
  assert.equal(nudgeBrightness(100, BRIGHTNESS_STEP), 100)
  assert.equal(nudgeBrightness(47, BRIGHTNESS_STEP), 52)
  assert.equal(clampBrightness(-3), 0)
  assert.equal(clampBrightness(140), 100)
})

test('pad e teclado Mac apontam para o mesmo delta', () => {
  assert.equal(fromPad('bright_down'), 'bright_down')
  assert.equal(fromPad('bright_up'), 'bright_up')
  assert.equal(fromEventCode('Comma'), 'bright_down')
  assert.equal(fromEventCode('Period'), 'bright_up')
  assert.equal(deltaFromPad('bright_down'), -BRIGHTNESS_STEP)
  assert.equal(deltaFromPad('bright_up'), BRIGHTNESS_STEP)
})

test('pad e código desconhecidos são ignorados', () => {
  const junk = faker.string.alpha({ length: { min: 6, max: 16 } })
  assert.equal(fromPad(junk), null)
  assert.equal(fromEventCode(junk), null)
  assert.equal(deltaFromPad(junk), null)
})
