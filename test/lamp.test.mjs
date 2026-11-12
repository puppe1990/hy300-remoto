import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import {
  LAMP_EN_PATH,
  LAMP_OFF_VALUE,
  LAMP_ON_VALUE,
  LAMP_POWER_PATH,
  fromPad,
  lampOnFromPad,
  parseLamp,
} from '../web/js/lamp.mjs'

beforeEach(() => {
  faker.seed(20261112)
})

test('GPIO Lcd_power 0 é tela off; 255 é on', () => {
  assert.equal(LAMP_EN_PATH, '/sys/class/leds/Lcd_en/brightness')
  assert.equal(LAMP_POWER_PATH, '/sys/class/leds/Lcd_power/brightness')
  assert.equal(LAMP_ON_VALUE, 255)
  assert.equal(LAMP_OFF_VALUE, 0)
  assert.deepEqual(parseLamp('255\n'), { ok: true, on: true })
  assert.deepEqual(parseLamp('0'), { ok: true, on: false })
  assert.deepEqual(parseLamp(''), { ok: true, on: false })
})

test('parse recusa lixo e cita o recebido', () => {
  const junk = `abc${faker.number.int({ min: 1, max: 999 })}`
  const got = parseLamp(junk)
  assert.equal(got.ok, false)
  assert.ok(got.error.includes(junk), got.error)
  assert.ok(got.error.includes('0 ou 255'), got.error)
})

test('pad Tela on/off', () => {
  assert.equal(fromPad('lamp_on'), 'lamp_on')
  assert.equal(fromPad('lamp_off'), 'lamp_off')
  assert.equal(lampOnFromPad('lamp_on'), true)
  assert.equal(lampOnFromPad('lamp_off'), false)
  assert.equal(fromPad(faker.string.alpha({ length: 8 })), null)
  assert.equal(lampOnFromPad(faker.string.alpha({ length: 8 })), null)
})
