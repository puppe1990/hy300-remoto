# HY300 wheel swipe Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Roda do mouse (e scroll de dois dedos) no preview manda `input swipe` vertical no HY300.

**Architecture:** Funções puras duplicadas em `web/js/swipe.mjs` e `src-tauri/src/swipe.rs` (goldens iguais). Trait `AdbDevice::swipe` + `swipe_args`. `app.js` cola `wheel` + coalesce 50 ms + lock. Tauri `send_swipe` via `adb_blocking`. Sem retry; `SCREENCAP_TIMEOUT` continua só no screencap.

**Tech Stack:** Tauri 2, vanilla `web/`, `node:test` + `@faker-js/faker`, Cargo `--lib --no-default-features` + crate `fake`.

Spec: `docs/superpowers/specs/2026-11-10-hy300-wheel-swipe-design.md`

---

## File map

| File                              | Role                                                   |
| --------------------------------- | ------------------------------------------------------ |
| Create `web/js/swipe.mjs`         | Dual JS: normalize, travel, `wheelToSwipe`, swipe lock |
| Create `test/swipe.test.mjs`      | Goldens JS                                             |
| Create `src-tauri/src/swipe.rs`   | Dual Rust + testes `fake`                              |
| Modify `src-tauri/src/lib.rs`     | `pub mod swipe`                                        |
| Modify `src-tauri/src/adb/mod.rs` | `swipe_args`, trait, `CliAdb`, `FakeAdb`               |
| Modify `src-tauri/src/main.rs`    | comando `send_swipe`                                   |
| Modify `web/js/app.js`            | listener `wheel`, burst 50 ms, invoke                  |
| Modify `web/index.html`           | dica do rodapé                                         |
| Modify `AGENTS.md`                | caveat da roda + dual swipe                            |

Working directory: `produtos/hy300-remoto`.

---

### Task 1: JS wheel → swipe

**Files:**

- Create: `test/swipe.test.mjs`
- Create: `web/js/swipe.mjs`

- [ ] **Step 1: Write the failing test**

Create `test/swipe.test.mjs`:

```javascript
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

test('flick satura a viagem em 240', () => {
  const swipe = wheelToSwipe(80, CENTER, HY300_SCREEN)
  assert.equal(swipe.y2, CENTER.y - 240)
  assert.equal(swipe.x1, swipe.x2)
})

test('origem ausente é null', () => {
  assert.equal(wheelToSwipe(30, null, HY300_SCREEN), null)
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
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cd /Users/matheuspuppe/Desktop/Projetos/produtos/hy300-remoto && node --test test/swipe.test.mjs
```

Expected: FAIL, `ERR_MODULE_NOT_FOUND` for `../web/js/swipe.mjs`.

- [ ] **Step 3: Write minimal implementation**

Create `web/js/swipe.mjs`:

```javascript
/** Dual of `src-tauri/src/swipe.rs` — keep goldens in both suites. */
export const SWIPE_COALESCE_MS = 50
export const SWIPE_DURATION_MS = 200
export const SWIPE_MIN_TRAVEL = 120
export const SWIPE_MAX_TRAVEL = 240

/**
 * Convert a WheelEvent deltaY into pixels. Unknown deltaMode is ignored.
 * @param {number} deltaY
 * @param {number} deltaMode
 * @returns {number|null}
 * @example
 * normalizeWheelDelta(2, 1) // 32
 */
export function normalizeWheelDelta(deltaY, deltaMode) {
  if (deltaMode === 0) return Number(deltaY)
  if (deltaMode === 1) return Number(deltaY) * 16
  if (deltaMode === 2) return Number(deltaY) * 400
  return null
}

/**
 * Finger travel in pixels. One notch saturates at 120; a flick at 240.
 * @param {number} absNormalizedDelta
 * @returns {number}
 * @example
 * swipeTravel(4) // 120
 */
export function swipeTravel(absNormalizedDelta) {
  const raw = Math.round(Math.abs(Number(absNormalizedDelta)) * 4)
  if (raw < SWIPE_MIN_TRAVEL) return SWIPE_MIN_TRAVEL
  if (raw > SWIPE_MAX_TRAVEL) return SWIPE_MAX_TRAVEL
  return raw
}

/**
 * Map normalized wheel delta + origin pixel to a vertical `input swipe`.
 * `deltaY > 0` (scroll down) moves the finger up. Origin null → null.
 * @param {number} normalizedDeltaY
 * @param {{x: number, y: number}|null} origin
 * @param {{width: number, height: number}} screen
 * @returns {{x1: number, y1: number, x2: number, y2: number, durationMs: number}|null}
 * @example
 * wheelToSwipe(30, { x: 640, y: 360 }, { width: 1280, height: 720 }).y2 // 240
 */
export function wheelToSwipe(normalizedDeltaY, origin, screen) {
  if (origin == null) return null
  if (!Number.isFinite(normalizedDeltaY) || normalizedDeltaY === 0) return null
  if (!screen || screen.width === 0 || screen.height === 0) return null
  const travel = swipeTravel(Math.abs(normalizedDeltaY))
  const dir = normalizedDeltaY > 0 ? -1 : 1
  const x1 = origin.x
  const y1 = origin.y
  const maxY = screen.height - 1
  const y2 = Math.min(maxY, Math.max(0, y1 + dir * travel))
  if (y2 === y1) return null
  return { x1, y1, x2: x1, y2, durationMs: SWIPE_DURATION_MS }
}

/**
 * Acquire the in-flight swipe lock. False if a swipe is already running.
 * @param {{inFlight: boolean}} state
 * @returns {boolean}
 * @example
 * takeSwipeLock({ inFlight: false }) // true
 */
export function takeSwipeLock(state) {
  if (state.inFlight) return false
  state.inFlight = true
  return true
}

/**
 * Drop the in-flight swipe lock so a coalesced burst can fire.
 * @param {{inFlight: boolean}} state
 * @returns {void}
 */
export function releaseSwipeLock(state) {
  state.inFlight = false
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
cd /Users/matheuspuppe/Desktop/Projetos/produtos/hy300-remoto && node --test test/swipe.test.mjs
```

Expected: PASS, 9 tests.

- [ ] **Step 5: Commit**

```bash
git add test/swipe.test.mjs web/js/swipe.mjs
git commit -m "feat: map wheel delta to vertical ADB swipe in JS"
```

---

### Task 2: Rust wheel → swipe

**Files:**

- Create: `src-tauri/src/swipe.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add `pub mod swipe;` to `src-tauri/src/lib.rs` after `pub mod tap;`.

Create `src-tauri/src/swipe.rs` with tests calling functions that do not exist yet (file can contain only the `#[cfg(test)]` module first — cargo will fail to compile). Put this as the whole file so the next step replaces it with tests+impl; for the red step, include the test module and empty `todo!` stubs:

```rust
//! Dual of `web/js/swipe.mjs` — keep goldens in both suites.

use crate::tap::ScreenSize;

pub const SWIPE_COALESCE_MS: u64 = 50;
pub const SWIPE_DURATION_MS: u32 = 200;
pub const SWIPE_MIN_TRAVEL: i32 = 120;
pub const SWIPE_MAX_TRAVEL: i32 = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Swipe {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub duration_ms: u32,
}

pub fn normalize_wheel_delta(_delta_y: f64, _delta_mode: u32) -> Option<f64> {
    todo!("normalize_wheel_delta")
}

pub fn swipe_travel(_abs_normalized_delta: f64) -> i32 {
    todo!("swipe_travel")
}

pub fn wheel_to_swipe(
    _normalized_delta_y: f64,
    _origin: Option<(i32, i32)>,
    _screen: ScreenSize,
) -> Option<Swipe> {
    todo!("wheel_to_swipe")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tap::HY300_SCREEN;
    use fake::Fake;

    const CENTER: (i32, i32) = (640, 360);

    #[test]
    fn constants_match_js() {
        assert_eq!(SWIPE_COALESCE_MS, 50);
        assert_eq!(SWIPE_DURATION_MS, 200);
    }

    #[test]
    fn delta_mode_normalizes() {
        let dy: f64 = (1u32..10u32).fake::<u32>() as f64;
        assert_eq!(normalize_wheel_delta(dy, 0), Some(dy));
        assert_eq!(normalize_wheel_delta(dy, 1), Some(dy * 16.0));
        assert_eq!(normalize_wheel_delta(dy, 2), Some(dy * 400.0));
        assert_eq!(normalize_wheel_delta(dy, 99), None);
    }

    #[test]
    fn travel_clamps_120_240() {
        assert_eq!(swipe_travel(4.0), 120);
        assert_eq!(swipe_travel(60.0), 240);
    }

    #[test]
    fn scroll_down_at_center_moves_finger_up() {
        let swipe = wheel_to_swipe(30.0, Some(CENTER), HY300_SCREEN).unwrap();
        assert_eq!(
            swipe,
            Swipe {
                x1: 640,
                y1: 360,
                x2: 640,
                y2: 240,
                duration_ms: 200,
            }
        );
    }

    #[test]
    fn flick_saturates_travel_at_240() {
        let swipe = wheel_to_swipe(80.0, Some(CENTER), HY300_SCREEN).unwrap();
        assert_eq!(swipe.y2, 120);
        assert_eq!(swipe.x1, swipe.x2);
    }

    #[test]
    fn missing_origin_is_none() {
        assert_eq!(wheel_to_swipe(30.0, None, HY300_SCREEN), None);
    }

    #[test]
    fn top_edge_cannot_swipe_up() {
        assert_eq!(
            wheel_to_swipe(30.0, Some((640, 0)), HY300_SCREEN),
            None
        );
    }

    #[test]
    fn bottom_edge_cannot_swipe_down() {
        assert_eq!(
            wheel_to_swipe(-30.0, Some((640, 719)), HY300_SCREEN),
            None
        );
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cd /Users/matheuspuppe/Desktop/Projetos/produtos/hy300-remoto && cargo test --manifest-path src-tauri/Cargo.toml --lib --no-default-features swipe -- --nocapture
```

Expected: FAIL, panic `not yet implemented: normalize_wheel_delta` (or the first `todo!` the suite hits).

- [ ] **Step 3: Write minimal implementation**

Replace the three `todo!` functions in `src-tauri/src/swipe.rs` with:

````rust
/// Convert a WheelEvent deltaY into pixels. Unknown deltaMode is ignored.
///
/// ```
/// use hy300_remoto::swipe::normalize_wheel_delta;
/// assert_eq!(normalize_wheel_delta(2.0, 1), Some(32.0));
/// ```
pub fn normalize_wheel_delta(delta_y: f64, delta_mode: u32) -> Option<f64> {
    match delta_mode {
        0 => Some(delta_y),
        1 => Some(delta_y * 16.0),
        2 => Some(delta_y * 400.0),
        _ => None,
    }
}

/// Finger travel in pixels. One notch saturates at 120; a flick at 240.
pub fn swipe_travel(abs_normalized_delta: f64) -> i32 {
    let raw = (abs_normalized_delta.abs() * 4.0).round() as i32;
    raw.clamp(SWIPE_MIN_TRAVEL, SWIPE_MAX_TRAVEL)
}

/// Map normalized wheel delta + origin pixel to a vertical `input swipe`.
/// `delta_y > 0` (scroll down) moves the finger up.
pub fn wheel_to_swipe(
    normalized_delta_y: f64,
    origin: Option<(i32, i32)>,
    screen: ScreenSize,
) -> Option<Swipe> {
    let (x1, y1) = origin?;
    if normalized_delta_y == 0.0 || !normalized_delta_y.is_finite() {
        return None;
    }
    if screen.width == 0 || screen.height == 0 {
        return None;
    }
    let travel = swipe_travel(normalized_delta_y.abs());
    let dir = if normalized_delta_y > 0.0 { -1 } else { 1 };
    let max_y = screen.height.saturating_sub(1) as i32;
    let y2 = (y1 + dir * travel).clamp(0, max_y);
    if y2 == y1 {
        return None;
    }
    Some(Swipe {
        x1,
        y1,
        x2: x1,
        y2,
        duration_ms: SWIPE_DURATION_MS,
    })
}
````

Keep the constants, `Swipe` struct, and `#[cfg(test)]` module from Step 1.

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
cd /Users/matheuspuppe/Desktop/Projetos/produtos/hy300-remoto && cargo test --manifest-path src-tauri/Cargo.toml --lib --no-default-features swipe -- --nocapture
```

Expected: PASS, 8 tests in `swipe`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/swipe.rs
git commit -m "feat: map wheel delta to vertical ADB swipe in Rust"
```

---

### Task 3: ADB `swipe` argv + FakeAdb

**Files:**

- Modify: `src-tauri/src/adb/mod.rs`

- [ ] **Step 1: Write the failing test**

In `src-tauri/src/adb/mod.rs`, add to `argv_uses_serial_and_input` after the tap asserts:

```rust
        let swipe = swipe_args(&host, 640, 360, 640, 240, 200);
        assert_eq!(swipe[4], "swipe");
        assert_eq!(swipe[5], "640");
        assert_eq!(swipe[6], "360");
        assert_eq!(swipe[7], "640");
        assert_eq!(swipe[8], "240");
        assert_eq!(swipe[9], "200");
```

In `fake_records_key_and_tap`, after `fake.tap(10, 20).unwrap();`:

```rust
        fake.swipe(640, 360, 640, 240, 200).unwrap();
        assert_eq!(*fake.swipes.borrow(), vec![(640, 360, 640, 240, 200)]);
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cd /Users/matheuspuppe/Desktop/Projetos/produtos/hy300-remoto && cargo test --manifest-path src-tauri/Cargo.toml --lib --no-default-features adb -- --nocapture
```

Expected: FAIL, cannot find `swipe_args` / `swipe` / `swipes`.

- [ ] **Step 3: Write minimal implementation**

Add to the trait after `fn tap`:

```rust
    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<(), AdbError>;
```

Add after `tap_args`:

```rust
pub fn swipe_args(
    host: &str,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    duration_ms: u32,
) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "input".into(),
        "swipe".into(),
        x1.to_string(),
        y1.to_string(),
        x2.to_string(),
        y2.to_string(),
        duration_ms.to_string(),
    ]
}
```

In `impl AdbDevice for CliAdb`, after `tap`:

```rust
    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<(), AdbError> {
        self.run(&swipe_args(&self.host, x1, y1, x2, y2, duration_ms))
            .map(|_| ())
    }
```

`CliAdb::run` already has no timeout. Do not pass `SCREENCAP_TIMEOUT` here.

In `FakeAdb` add:

```rust
    pub swipes: RefCell<Vec<(i32, i32, i32, i32, u32)>>,
```

In `impl AdbDevice for FakeAdb`, after `tap`:

```rust
    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<(), AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        self.swipes
            .borrow_mut()
            .push((x1, y1, x2, y2, duration_ms));
        Ok(())
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
cd /Users/matheuspuppe/Desktop/Projetos/produtos/hy300-remoto && cargo test --manifest-path src-tauri/Cargo.toml --lib --no-default-features --quiet
```

Expected: PASS, previous 24 + new swipe tests (lib ~32).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/adb/mod.rs
git commit -m "feat: add AdbDevice swipe argv and FakeAdb log"
```

---

### Task 4: Tauri `send_swipe`

**Files:**

- Modify: `src-tauri/src/main.rs`

No `--lib` test covers `main.rs`. After the command exists, compile the desktop binary so `.into()`-style Tauri ambiguity cannot ship again.

- [ ] **Step 1: Write the command**

After `send_tap` in `src-tauri/src/main.rs`:

```rust
#[tauri::command]
async fn send_swipe(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    duration_ms: u32,
    session: State<'_, AdbSession>,
) -> Result<(), String> {
    let adb = session.adb()?;
    adb_blocking(
        move || {
            adb.swipe(x1, y1, x2, y2, duration_ms)
                .map_err(|e| e.message)
        },
        "adb",
    )
    .await
}
```

Add `send_swipe` to `tauri::generate_handler!` after `send_tap`.

- [ ] **Step 2: Compile desktop binary**

Run:

```bash
cd /Users/matheuspuppe/Desktop/Projetos/produtos/hy300-remoto/src-tauri && cargo build --features desktop
```

Expected: `Finished dev profile` with no errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: expose send_swipe Tauri command"
```

---

### Task 5: Preview wheel glue + hint

**Files:**

- Modify: `web/js/app.js`
- Modify: `web/index.html`

- [ ] **Step 1: Wire the wheel listener**

Replace the import block at the top of `web/js/app.js` with:

```javascript
import { fromEventCode, keycode } from './keys.mjs'
import { mapTap } from './tap.mjs'
import { parseHost } from './host.mjs'
import { PREVIEW_INTERVAL_MS, releasePreviewLock, statusCopy, takePreviewLock } from './preview.mjs'
import {
  SWIPE_COALESCE_MS,
  normalizeWheelDelta,
  releaseSwipeLock,
  takeSwipeLock,
  wheelToSwipe,
} from './swipe.mjs'
```

After `const previewLock = { inFlight: false }` add:

```javascript
const swipeLock = { inFlight: false }
const swipeBurst = { accum: 0, clientX: 0, clientY: 0, timer: 0 }
```

After `onPreviewClick` add:

```javascript
function onPreviewWheel(event) {
  event.preventDefault()
  const norm = normalizeWheelDelta(event.deltaY, event.deltaMode)
  if (norm == null) return
  swipeBurst.clientX = event.clientX
  swipeBurst.clientY = event.clientY
  if (norm !== 0) swipeBurst.accum += norm
  if (!swipeBurst.timer) {
    swipeBurst.timer = window.setTimeout(flushSwipe, SWIPE_COALESCE_MS)
  }
}

function flushSwipe() {
  swipeBurst.timer = 0
  void sendAccumulatedSwipe()
}

function plannedFromBurst() {
  if (swipeBurst.accum === 0) return null
  const rect = wrapEl.getBoundingClientRect()
  const origin = mapTap(swipeBurst.clientX - rect.left, swipeBurst.clientY - rect.top, rect, screen)
  if (!origin) {
    swipeBurst.accum = 0
    return null
  }
  const planned = wheelToSwipe(swipeBurst.accum, origin, screen)
  if (!planned) {
    swipeBurst.accum = 0
    return null
  }
  return planned
}

async function sendAccumulatedSwipe() {
  if (!invoke) return
  const planned = plannedFromBurst()
  if (!planned) return
  if (!takeSwipeLock(swipeLock)) return
  swipeBurst.accum = 0
  try {
    await invoke('send_swipe', planned)
    setStatus('connected')
  } catch (err) {
    setStatus('error', String(err))
  } finally {
    releaseSwipeLock(swipeLock)
    await sendAccumulatedSwipe()
  }
}
```

After `wrapEl.addEventListener('click', onPreviewClick)` add:

```javascript
wrapEl.addEventListener('wheel', onPreviewWheel, { passive: false })
```

In `web/index.html` replace the hint line with:

```html
<p class="hint keys-hint">setas · Enter · Esc · H · +/− · roda no preview</p>
```

- [ ] **Step 2: Run JS tests (glue has no extra unit file)**

Run:

```bash
cd /Users/matheuspuppe/Desktop/Projetos/produtos/hy300-remoto && node --test test/*.test.mjs
```

Expected: PASS, previous 14 + 9 swipe = 23.

- [ ] **Step 3: Commit**

```bash
git add web/js/app.js web/index.html
git commit -m "feat: swipe HY300 from preview wheel"
```

---

### Task 6: AGENTS.md + CI gate

**Files:**

- Modify: `AGENTS.md`

- [ ] **Step 1: Document the wheel**

In `AGENTS.md` Caveats, after the preview/tap bullet, add:

```markdown
- Roda no preview (e scroll de dois dedos) manda `input swipe` vertical: coalesce 50 ms, viagem 120–240 px, duração 200 ms. Letterbox ignora e zera o accum. `deltaX` fica de fora. Lock próprio, separado do screencap. Sem timeout extra no swipe.
```

Change the dual-maps bullet to:

```markdown
- Mapas de tecla, tap e swipe duplicados em Rust e `web/js/*.mjs` — goldens iguais. Do not merge them.
```

In Code style, change “Dual key/tap maps” to “Dual key/tap/swipe maps are the allowed duplication.”

In Structure, mention `swipe.rs` / `swipe.mjs` next to the other duals.

- [ ] **Step 2: Run the full gate**

Run:

```bash
cd /Users/matheuspuppe/Desktop/Projetos/produtos/hy300-remoto && cargo fmt --manifest-path src-tauri/Cargo.toml && npm run ci && cargo build --manifest-path src-tauri/Cargo.toml --features desktop
```

Expected: Prettier/ESLint clean; 23 JS tests pass; cargo lib tests pass; desktop binary compiles.

- [ ] **Step 3: Commit**

```bash
git add AGENTS.md
git commit -m "docs: note preview wheel as vertical ADB swipe"
```

---

## Self-review

**Spec coverage**

| Spec                                                                     | Task                                       |
| ------------------------------------------------------------------------ | ------------------------------------------ |
| `wheel` on `#preview-wrap`, `{ passive: false }`, `preventDefault`       | 5                                          |
| `mapTap` origin, last event of 50 ms burst                               | 5                                          |
| letterbox / `mapTap` null → no ADB, zero accum                           | 1 (`origin` null) + 5 (`plannedFromBurst`) |
| `deltaY === 0` no ADB                                                    | 1 `wheelToSwipe`                           |
| `deltaMode` 0/1/2 / other                                                | 1 + 2                                      |
| `SWIPE_COALESCE_MS = 50`                                                 | 1, 2, 5                                    |
| `deltaY > 0` → `y2 < y1`                                                 | 1 + 2                                      |
| travel `clamp(round(\|accum\| * 4), 120, 240)`                           | 1 + 2                                      |
| duration 200                                                             | 1 + 2 + 3 argv                             |
| `y2 === y1` → no ADB                                                     | 1 + 2 edge tests                           |
| swipe lock ≠ screencap lock; one in-flight; accum while busy; fire after | 1 lock helpers + 5 `sendAccumulatedSwipe`  |
| no retry, no extra timeout                                               | 3 `CliAdb::run`; 5 no retry                |
| `input swipe x1 y1 x2 y2 200`                                            | 3                                          |
| `AdbDevice::swipe`, `FakeAdb.swipes`                                     | 3                                          |
| `send_swipe` + `adb_blocking`                                            | 4                                          |
| dual `swipe.mjs` / `swipe.rs` + JSDoc/rustdoc                            | 1 + 2                                      |
| footer hint                                                              | 5                                          |
| AGENTS.md                                                                | 6                                          |
| `npm test` / no real adb                                                 | all tasks                                  |

**Placeholders:** none.

**Types:** `Swipe` / `{ x1, y1, x2, y2, durationMs }` / `duration_ms: u32` / `FakeAdb.swipes: (i32,i32,i32,i32,u32)` stay the same from Task 1 through Task 5. `takeSwipeLock` / `releaseSwipeLock` match the preview lock shape `{ inFlight: boolean }`.
