# HY300 Remoto — roda no preview = swipe vertical

Data: 2026-11-10  
Pasta: `produtos/hy300-remoto`  
Status: approved

Incremento da spec `2026-11-10-hy300-remoto-design.md`. Aqui a roda do mouse (e o scroll de dois dedos do trackpad, mesmo evento `wheel`) em cima do preview manda `input swipe` no HY300.

Substitui, só neste recorte, o “Sem swipe” / “Fora do v1: Swipe / arraste” da spec original. Arraste com o botão do mouse e swipe horizontal continuam fora.

## Objetivo

No preview, girar a roda pra baixo faz o conteúdo da tela do projetor descer (dedo sobe). Roda pra cima sobe o conteúdo. Um aparelho, testes sem ADB real.

## Fora deste ciclo

- Arraste / pointer swipe
- Swipe horizontal (`deltaX` ignora, mesmo se for maior que `deltaY`)
- `input roll` e `input motionevent`
- Mapear roda para DPAD ou PAGE_DOWN
- scrcpy, timeout extra, retry, circuit breaker
- Mudar o intervalo de 500 ms do screencap

## Evento

Listener `wheel` em `#preview-wrap` com `{ passive: false }`. `preventDefault()` sempre no wrap, pra janela do Mac não rolar.

Origem do swipe: o mesmo `mapTap(clientX − wrapLeft, clientY − wrapTop, wrapRect, screen)` do clique. Cursor da rajada: último `wheel` do burst de 50 ms. Se esse ponto cai no letterbox (ou `mapTap` devolve `null`), não manda ADB e zera o accum.

`deltaY === 0` → ignora (não chama ADB).

Normalizar antes de acumular:

| `deltaMode` | pixels acumulados |
| ----------- | ----------------- |
| 0 (pixel)   | `deltaY`          |
| 1 (line)    | `deltaY * 16`     |
| 2 (page)    | `deltaY * 400`    |
| outro       | ignora o evento   |

Constante: `SWIPE_COALESCE_MS = 50`.

## Direção e viagem

Convenção do browser: `deltaY > 0` = usuário quer o conteúdo descer = dedo swipe **pra cima** (`y2 < y1`). `deltaY < 0` = swipe pra baixo (`y2 > y1`).

Depois do coalesce, `accum` é a soma dos deltas normalizados.

```
travel = clamp(round(|accum| * 4), 120, 240)
dir    = accum > 0 ? -1 : +1
x1 = x2 = origin.x
y1 = origin.y
y2 = clamp(y1 + dir * travel, 0, screen.height - 1)
durationMs = 200
```

Tela padrão 1280×720 (a do frame, como o tap). Se `y2 === y1` (já na borda naquela direção) → não chama ADB.

Um notch típico (`|accum|` pequeno) satura em 120 px. Flick de trackpad satura em 240 px.

## Lock

Lock próprio, separado do lock do screencap (preview continua a 2 fps).

- Um `input swipe` por vez.
- Enquanto o swipe está no ar, novos `wheel` só acumulam.
- Quando o invoke volta (ok ou erro), se `accum !== 0` e o cursor ainda mapeia, dispara o próximo; senão zera.

Sem fila de N swipes. Sem retry.

## ADB

```
adb -s {host} shell input swipe {x1} {y1} {x2} {y2} 200
```

Trait `AdbDevice` ganha:

```
fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<(), AdbError>
```

`CliAdb` monta argv e chama `run` sem timeout (swipe é curto; `SCREENCAP_TIMEOUT` 4s continua só no screencap).

`FakeAdb` grava `swipes: RefCell<Vec<(i32, i32, i32, i32, u32)>>`.

Comando Tauri `send_swipe` via `adb_blocking(..., "adb")`, igual ao tap. Erro inclui argv e a saída do processo.

## Dual mapa

Mesmo padrão tecla/tap: funções puras duplicadas com goldens iguais.

- `web/js/swipe.mjs`
- `src-tauri/src/swipe.rs`

Superfície pública:

- `SWIPE_COALESCE_MS = 50`
- `SWIPE_DURATION_MS = 200`
- `normalizeWheelDelta(deltaY, deltaMode) -> number | null`
- `swipeTravel(absNormalizedDelta) -> number` (120–240)
- `wheelToSwipe(normalizedDeltaY, origin, screen) -> { x1, y1, x2, y2, durationMs } | null`

JSDoc / rustdoc com um exemplo. `app.js` só cola o `wheel`, o timer de 50 ms e o invoke.

## UI

Sem botão novo. A dica no rodapé passa a:

`setas · Enter · Esc · H · +/− · roda no preview`

## Testes

`npm test` (JS + `cargo test --lib --no-default-features`). `@faker-js/faker` e crate `fake`. Sem binário `adb` na CI.

Cobrir:

- `deltaY > 0` no centro 1280×720 → `y2 < y1`, `x1 === x2`, `durationMs === 200`
- `deltaY` pequeno → travel 120; `|accum| * 4 ≥ 240` → travel 240
- letterbox (mesmo caso do tap) → `null`
- origem `y = 0` e `deltaY > 0` → `null` (não tem pra onde subir o dedo)
- origem `y = 719` e `deltaY < 0` → `null`
- `deltaMode` 1 e 2 normalizam; modo desconhecido → `null`
- `swipe_args` tem `swipe` e `200`
- `FakeAdb` registra o tuple

## Arquivos

```
web/js/swipe.mjs
test/swipe.test.mjs
src-tauri/src/swipe.rs
src-tauri/src/lib.rs          pub mod swipe
src-tauri/src/adb/mod.rs      swipe_args + trait + FakeAdb + CliAdb
src-tauri/src/main.rs         send_swipe
web/js/app.js                 wheel + coalesce + lock
web/index.html                dica
AGENTS.md                     caveat da roda
```
