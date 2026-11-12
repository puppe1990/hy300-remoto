import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import { fromEventCode, fromPad, keycode } from '../web/js/keys.mjs'

beforeEach(() => {
  faker.seed(20261110)
})

test('pad ids batem com os keycodes AOSP', () => {
  assert.equal(keycode('up'), 19)
  assert.equal(keycode('down'), 20)
  assert.equal(keycode('left'), 21)
  assert.equal(keycode('right'), 22)
  assert.equal(keycode('ok'), 23)
  assert.equal(keycode('back'), 4)
  assert.equal(keycode('home'), 3)
  assert.equal(keycode('vol_up'), 24)
  assert.equal(keycode('vol_down'), 25)
  assert.equal(keycode('mute'), 164)
  assert.equal(keycode('power'), 26)
  assert.equal(keycode('rew'), 89)
  assert.equal(keycode('ff'), 90)
})

test('event.code aponta para o mesmo pad', () => {
  assert.equal(fromEventCode('ArrowUp'), 'up')
  assert.equal(fromEventCode('Enter'), 'ok')
  assert.equal(fromEventCode('NumpadEnter'), 'ok')
  assert.equal(fromEventCode('Escape'), 'back')
  assert.equal(fromEventCode('KeyH'), 'home')
  assert.equal(fromEventCode('Equal'), 'vol_up')
  assert.equal(fromEventCode('Minus'), 'vol_down')
  assert.equal(fromEventCode('KeyM'), 'mute')
  assert.equal(fromEventCode('BracketLeft'), 'rew')
  assert.equal(fromEventCode('BracketRight'), 'ff')
})

test('código e pad desconhecidos são ignorados', () => {
  const junk = faker.string.alpha({ length: { min: 6, max: 16 } })
  assert.equal(fromPad(junk), null)
  assert.equal(fromEventCode(junk), null)
  assert.equal(keycode(junk), null)
})
