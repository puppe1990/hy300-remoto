import assert from 'node:assert/strict'
import { beforeEach, test } from 'node:test'
import { faker } from '@faker-js/faker'
import {
  MOUSE_COALESCE_MS,
  escapeExitsMouseMode,
  invalidMouseAction,
  parseMouseAction,
  shouldSkipDuplicateMove,
  toggleMouseMode,
} from '../web/js/mouse.mjs'

beforeEach(() => {
  faker.seed(20261112)
})

test('coalesce é 50 ms', () => {
  assert.equal(MOUSE_COALESCE_MS, 50)
})

test('parse aceita MOVE DOWN UP e recusa o resto', () => {
  assert.equal(parseMouseAction('MOVE'), 'MOVE')
  assert.equal(parseMouseAction('DOWN'), 'DOWN')
  assert.equal(parseMouseAction('UP'), 'UP')
  assert.equal(parseMouseAction('move'), null)
  assert.equal(parseMouseAction('Tap'), null)
  assert.equal(parseMouseAction('HOVER'), null)
  assert.equal(parseMouseAction(''), null)
  const junk = faker.string.alpha({ length: 8 })
  assert.equal(parseMouseAction(junk), null)
  const err = invalidMouseAction(junk)
  assert.ok(err.includes(junk), err)
  assert.ok(err.includes('MOVE|DOWN|UP'), err)
})

test('pula MOVE no mesmo pixel do último evento', () => {
  const x = faker.number.int({ min: 0, max: 1279 })
  const y = faker.number.int({ min: 0, max: 719 })
  const move = { action: 'MOVE', x, y }
  assert.equal(shouldSkipDuplicateMove(null, move), false)
  assert.equal(shouldSkipDuplicateMove(move, move), true)
  assert.equal(shouldSkipDuplicateMove({ action: 'DOWN', x, y }, move), true)
  assert.equal(shouldSkipDuplicateMove(move, { action: 'MOVE', x: x + 1, y }), false)
  assert.equal(shouldSkipDuplicateMove(move, { action: 'DOWN', x, y }), false)
  assert.equal(shouldSkipDuplicateMove(move, { action: 'UP', x, y }), false)
})

test('toggle inverte o modo', () => {
  assert.equal(toggleMouseMode(false), true)
  assert.equal(toggleMouseMode(true), false)
})

test('Esc com modo ligado consome e desliga', () => {
  assert.deepEqual(escapeExitsMouseMode(true, 'Escape'), { on: false, consume: true })
  assert.deepEqual(escapeExitsMouseMode(false, 'Escape'), { on: false, consume: false })
  assert.deepEqual(escapeExitsMouseMode(true, 'KeyH'), { on: true, consume: false })
})
