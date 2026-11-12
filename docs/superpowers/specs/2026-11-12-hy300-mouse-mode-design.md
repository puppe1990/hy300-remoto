# HY300 Remoto — modo mouse no visor 1:1

Data: 2026-11-12  
Pasta: `produtos/hy300-remoto`  
Status: approved

Incremento da spec `2026-11-10-hy300-remoto-design.md`. Com o modo mouse ligado, o ponteiro do Mac em cima do visor dirige o cursor SOURCE_MOUSE do HY300 no mesmo pixel (`mapTap`). Clique e arraste viram `DOWN` / `MOVE` / `UP`, não `input tap`.

A spec da roda (`2026-11-10-hy300-wheel-swipe-design.md`) deixa `input roll` e `input motionevent` fora daquele recorte. Este ciclo abre só `input mouse motionevent` (fonte mouse). Touchscreen motionevent, `input roll` e mouse capturado fora da janela continuam fora.

Probe 2026-11-12 no aparelho `192.168.15.4:3268`: `persist.sys.txcz.nfmouse=1`; `adb shell input mouse motionevent MOVE 640 360` rc 0; `dumpsys input` registrou source `0x00002002` com `xCursorPosition=640.0 yCursorPosition=360.0`.

## Objetivo

No pad, um botão **Mouse**. Ligado: mover o ponteiro do Mac sobre o visor move o cursor do projetor no mesmo ponto da parede; clique primário clica como mouse; arraste segura o botão. Desligado: o visor volta a mandar `input tap` como hoje. Um aparelho. Testes sem ADB real.

## Fora deste ciclo

- Clique direito, botão do meio, scroll como mouse (`input roll`)
- `input motionevent` sem o source `mouse` (touchscreen)
- Mouse capturado / pointer lock fora da janela (scrcpy)
- Gadget HID / USB mouse virtual
- Gravar o modo em `host.json` ou outro arquivo
- Ligar/desligar `persist.sys.txcz.nfmouse` (já está `1`)
- Pintar uma seta extra no visor do Mac (o ponteiro do sistema fica)
- Mexer em `daemon12138`, GPIO, KEYCODE_POWER, intervalo de 500 ms do screencap

Risco aceito: o firmware pode não desenhar a setinha na parede sem mouse USB; o dispatcher ainda recebe SOURCE_MOUSE. Clique e hover valem mesmo assim.

## Modo

Estado só na sessão, default **desligado**. Não persiste.

| Entrada                                       | Modo desligado                       | Modo ligado                           |
| --------------------------------------------- | ------------------------------------ | ------------------------------------- |
| Botão **Mouse**                               | liga                                 | desliga                               |
| Esc (teclado Mac)                             | Voltar (`KEYCODE_BACK` 4), como hoje | desliga o modo; **não** manda BACK    |
| Botão Voltar do pad                           | BACK                                 | BACK                                  |
| `pointerdown` primário no conteúdo do visor   | `send_tap`                           | `mouse DOWN`                          |
| `pointermove` no conteúdo                     | ignora                               | `mouse MOVE` (coalesce)               |
| `pointerup` / `pointercancel` com botão preso | ignora                               | `mouse UP`                            |
| Roda no visor                                 | swipe vertical                       | swipe vertical (igual à spec da roda) |
| Letterbox                                     | ignora                               | ignora                                |

Não existe atalho de teclado para **entrar** no modo. Só o botão Mouse.

Campo host focado: teclado não muda o modo nem manda keyevent (já é a regra do pad).

Botão primário só (`previewPointerIsPrimary`). Outro botão no visor ignora.

## Mapa

O mesmo `mapTap` / `tapTargetBox` do toque. Resolução do frame (fallback 1280×720). Letterbox (`mapTap` `null`) não chama ADB.

`pointerup` / `pointercancel` fora do conteúdo, com o botão do mouse ainda preso neste gesto: `UP` no último pixel in-content enviado. Se o gesto nunca teve `DOWN` válido, não manda `UP`.

`setPointerCapture` no `#preview-wrap` no `DOWN` válido; solta no `UP` / `pointercancel`.

`pointercancel` manda `UP` (não `CANCEL`). `CANCEL` fica fora.

## Ações

Três strings, maiúsculas, iguais ao CLI Android:

`MOVE` | `DOWN` | `UP`

`parseMouseAction(value)` devolve uma dessas ou `null`. `move`, `Tap`, `HOVER`, vazio e qualquer outra coisa → `null`. A mensagem de erro cita o valor recebido e a forma esperada (`MOVE|DOWN|UP`).

`DOWN` e `UP` sempre enviam, mesmo no mesmo pixel do evento anterior. `MOVE` no mesmo `(x, y)` do último mouse enviado (qualquer ação) → não chama ADB.

## Cadência

Constante: `MOUSE_COALESCE_MS = 50`.

- `MOVE` entra no timer de 50 ms. Enquanto o invoke está no ar, novos `MOVE` só atualizam `x,y`. Ao voltar: se o pixel mudou e não é duplicata, dispara o próximo; senão para.
- `DOWN` / `UP` não usam o timer. Cancelam o timer de `MOVE` pendente. `DOWN` carrega a posição (não precisa flush de `MOVE` antes). `UP` manda um `MOVE` pendente diferente do último enviado e em seguida o `UP`.
- Lock próprio `{ inFlight }`, separado de screencap e swipe. Um motionevent por vez. Sem retry, sem timeout extra (`SCREENCAP_TIMEOUT` continua só no screencap).
- Slot `pendingClick`: no máximo um `DOWN` ou `UP` esperando o lock. Ao soltar o lock, `pendingClick` sai antes de um `MOVE` acumulado.

## ADB

```
adb -s {host} shell input mouse motionevent {MOVE|DOWN|UP} {x} {y}
```

Trait `AdbDevice` ganha:

```
fn mouse(&self, action: &str, x: i32, y: i32) -> Result<(), AdbError>
```

`CliAdb` recusa ação `null`/inválida antes de spawnar. `run` sem timeout. `FakeAdb` grava `mouse: RefCell<Vec<(String, i32, i32)>>`.

`mouse_args(host, action, x, y)` → `["-s", host, "shell", "input", "mouse", "motionevent", action, x, y]`.

Comando Tauri `send_mouse` via `adb_blocking(..., "adb")`, igual ao tap. Erro inclui argv e a saída do processo.

Não chama `setprop persist.sys.txcz.nfmouse`.

## Dual mapa

Mesmo padrão tecla/tap/swipe/lâmpada: funções puras duplicadas com goldens iguais.

- `web/js/mouse.mjs`
- `src-tauri/src/mouse.rs`

Superfície pública:

- `MOUSE_COALESCE_MS = 50`
- `parseMouseAction(value) -> 'MOVE'|'DOWN'|'UP'|null` / `parse_mouse_action`
- `shouldSkipDuplicateMove(last, next) -> bool` — `last` é `{ action, x, y } | null`; `next` é o candidato; `true` só quando `next.action === 'MOVE'` e `last` existe com o mesmo `(x, y)`
- `toggleMouseMode(on) -> bool` — `!on`
- `escapeExitsMouseMode(on, eventCode) -> { on: bool, consume: bool }` — `on && eventCode === 'Escape'` → `{ on: false, consume: true }`; senão `{ on, consume: false }`

JSDoc / rustdoc com um exemplo. `app.js` cola o toggle, os listeners do visor, o timer, o lock e o invoke. `mapTap` não se duplica aqui.

## UI

Nova fileira no pad, imediatamente abaixo de Tela off / Tela on:

```
<button type="button" id="mouse-toggle" aria-pressed="false">Mouse</button>
```

`aria-pressed` acompanha o modo. Classe `is-on` no botão quando ligado. `#preview-wrap` ganha `is-mouse` e `aria-label="Tela do projetor — mouse"` no modo ligado; volta ao label atual ao desligar. Cursor do wrap continua `crosshair` (não esconde o ponteiro do Mac).

Dica no rodapé:

`setas · Enter · Esc · H · +/− · M · [ ] · , . · roda no preview · Mouse`

## Testes

`npm test` (JS + `cargo test --lib --no-default-features`). `@faker-js/faker` e crate `fake`. Sem binário `adb` na CI. Sem falar com o projetor.

Cobrir:

- `parseMouseAction('MOVE'|'DOWN'|'UP')` devolve a ação; faker de string que não é essas três → `null`
- `shouldSkipDuplicateMove` no mesmo pixel MOVE→MOVE é `true`; pixel diferente é `false`; `last === null` é `false`; `DOWN`/`UP` no mesmo pixel é `false`
- `toggleMouseMode(false) === true`; `toggleMouseMode(true) === false`
- `escapeExitsMouseMode(true, 'Escape')` consome e desliga; modo off + Escape não consome; `KeyH` com modo on não consome
- `mouse_args` tem `mouse`, `motionevent`, a ação e o par faker `(x, y)`
- ação inválida na camada ADB/comando cita o valor e `MOVE|DOWN|UP`
- `FakeAdb` registra o tuple `(action, x, y)`
- letterbox não gera evento (reusa o caso do tap; o app não chama `send_mouse`)

## Arquivos

```
web/js/mouse.mjs
test/mouse.test.mjs
src-tauri/src/mouse.rs
src-tauri/src/lib.rs          pub mod mouse
src-tauri/src/adb/mod.rs      mouse_args + trait + FakeAdb + CliAdb
src-tauri/src/main.rs         send_mouse
web/js/app.js                 modo + pointermove/up/cancel + coalesce + lock
web/index.html                botão Mouse + dica
web/css/remote.css            .is-on no toggle / wrap is-mouse
AGENTS.md                     caveat do modo mouse
```
