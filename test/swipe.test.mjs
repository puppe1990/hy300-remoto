import assert from 'node:assert/strict'
import { test } from 'node:test'
import { faker } from '@faker-js/faker'
import { HY300_SCREEN } from '../web/js/tap.mjs'
import {
  SWIPE_COALESCE_MS,
  SWIPE_DURATION_MS,
  normalizeWheelDelta,
  releaseSwipeLock,
  swipeTravel,
  takeSwipeLock,
  wheelToSwipe,
} from '../web/js/swipe.mjs'

const CENTER = { x: HY300_SCREEN.width / 2, y: HY300_SCREEN.height / 2 }

test('constantes do coalesce e da duração', () => {
  assert.equal(SWIPE_COALESCE_MS, 50)
  assert.equal(SWIPE_DURATION_MS, 200)
})

test('deltaMode 0 é pixel; 1 linha; 2 página; outro null', () => {
  const dy = faker.number.int({ min: 1, max: 9 })
  assert.equal(normalizeWheelDelta(dy, 0), dy)
  assert.equal(normalizeWheelDelta(dy, 1), dy * 16)
  assert.equal(normalizeWheelDelta(dy, 2), dy * 400)
  assert.equal(normalizeWheelDelta(dy, 99), null)
})

test('viagem satura em 120 e 240', () => {
  assert.equal(swipeTravel(4), 120)
  assert.equal(swipeTravel(60), 240)
})

test('viagem linear no meio da faixa', () => {
  assert.equal(swipeTravel(40), 160)
})

test('roda pra baixo no centro sobe o dedo', () => {
  const swipe = wheelToSwipe(30, CENTER, HY300_SCREEN)
  assert.deepEqual(swipe, {
    x1: CENTER.x,
    y1: CENTER.y,
    x2: CENTER.x,
    y2: CENTER.y - 120,
    durationMs: 200,
  })
})

test('roda pra cima no centro desce o dedo', () => {
  const swipe = wheelToSwipe(-30, CENTER, HY300_SCREEN)
  assert.equal(swipe.y2, CENTER.y + 120)
})

test('flick satura a viagem em 240', () => {
  const swipe = wheelToSwipe(80, CENTER, HY300_SCREEN)
  assert.equal(swipe.y2, CENTER.y - 240)
  assert.equal(swipe.x1, swipe.x2)
})

test('origem ausente é null', () => {
  assert.equal(wheelToSwipe(30, null, HY300_SCREEN), null)
})

test('delta zero não manda swipe', () => {
  assert.equal(wheelToSwipe(0, CENTER, HY300_SCREEN), null)
})

test('borda de cima não swipe pra cima', () => {
  assert.equal(wheelToSwipe(30, { x: 640, y: 0 }, HY300_SCREEN), null)
})

test('borda de baixo não swipe pra baixo', () => {
  assert.equal(wheelToSwipe(-30, { x: 640, y: 719 }, HY300_SCREEN), null)
})

test('não dispara outro swipe enquanto um está no ar', () => {
  const state = { inFlight: false }
  assert.equal(takeSwipeLock(state), true)
  assert.equal(takeSwipeLock(state), false)
  releaseSwipeLock(state)
  assert.equal(takeSwipeLock(state), true)
})
