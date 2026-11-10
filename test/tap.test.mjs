import assert from 'node:assert/strict'
import { test } from 'node:test'
import { faker } from '@faker-js/faker'
import { HY300_SCREEN, mapTap } from '../web/js/tap.mjs'

test('clique no centro da caixa 1:1 acerta o pixel central', () => {
  const preview = { width: HY300_SCREEN.width, height: HY300_SCREEN.height }
  const hit = mapTap(preview.width / 2, preview.height / 2, preview, HY300_SCREEN)
  assert.deepEqual(hit, { x: HY300_SCREEN.width / 2, y: HY300_SCREEN.height / 2 })
})

test('origem é (0,0)', () => {
  const w = faker.number.int({ min: 320, max: 1920 })
  const h = faker.number.int({ min: 180, max: 1080 })
  const screen = { width: w, height: h }
  assert.deepEqual(mapTap(0, 0, { width: w, height: h }, screen), { x: 0, y: 0 })
})

test('letterbox de cima não manda tap', () => {
  assert.equal(mapTap(640, 10, { width: 1280, height: 900 }, HY300_SCREEN), null)
})

test('letterbox da lateral não manda tap', () => {
  assert.equal(mapTap(10, 360, { width: 1600, height: 720 }, HY300_SCREEN), null)
})

test('canto satura no último pixel', () => {
  const hit = mapTap(1280, 720, { width: 1280, height: 720 }, HY300_SCREEN)
  assert.deepEqual(hit, { x: 1279, y: 719 })
})
