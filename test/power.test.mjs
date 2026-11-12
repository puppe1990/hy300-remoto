import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import { POWER_OFF_FOCUS, combinePower, powerFromWindowDump } from '../web/js/power.mjs'

beforeEach(() => {
  faker.seed(20261112)
})

test('foco GlobalActions é lâmpada em standby, não SoC morto', () => {
  assert.equal(POWER_OFF_FOCUS, 'GlobalActions')
  const dump = `  mCurrentFocus=Window{c85f1b1 u0 LegacyGlobalActions}\n  mFocusedApp=ActivityRecord{dc5603d u0 com.bozee.usbdisplay/.StartupActivity t220}`
  assert.equal(powerFromWindowDump(dump), 'off')
})

test('foco de app no ar é ligado', () => {
  const pkg = faker.internet.domainWord()
  const dump = `  mCurrentFocus=Window{ac9b6f0 u0 com.${pkg}.usbdisplay/com.${pkg}.MainActivity}`
  assert.equal(powerFromWindowDump(dump), 'on')
})

test('dump vazio ou sem mCurrentFocus não inventa desligado', () => {
  assert.equal(powerFromWindowDump(''), 'on')
  assert.equal(powerFromWindowDump(faker.lorem.sentence()), 'on')
})

test('tela GPIO apagada ou diálogo de energia é desligado', () => {
  assert.equal(combinePower('on', true), 'on')
  assert.equal(combinePower('off', true), 'off')
  assert.equal(combinePower('on', false), 'off')
  assert.equal(combinePower('off', false), 'off')
})
