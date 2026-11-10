import assert from 'node:assert/strict'
import { test } from 'node:test'
import { faker } from '@faker-js/faker'
import {
  PREVIEW_INTERVAL_MS,
  releasePreviewLock,
  statusCopy,
  takePreviewLock,
} from '../web/js/preview.mjs'

test('intervalo é 2 fps', () => {
  assert.equal(PREVIEW_INTERVAL_MS, 500)
})

test('status em português', () => {
  assert.equal(statusCopy('connecting'), 'conectando…')
  assert.equal(statusCopy('connected'), 'ligado')
  assert.equal(statusCopy('disconnected'), 'sem conexão')
  const detail = faker.lorem.words(3)
  const copy = statusCopy('error', detail)
  assert.ok(copy.startsWith('falhou: '))
  assert.ok(copy.includes(detail))
})

test('não dispara outro screencap enquanto um está no ar', () => {
  const state = { inFlight: false }
  assert.equal(takePreviewLock(state), true)
  assert.equal(state.inFlight, true)
  assert.equal(takePreviewLock(state), false)
  releasePreviewLock(state)
  assert.equal(state.inFlight, false)
  assert.equal(takePreviewLock(state), true)
})
