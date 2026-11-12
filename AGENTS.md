# HY300 Remoto

Tauri 2 no Mac: preview + pad + teclado do projetor HY300 via ADB wireless. Identifier `com.gestaobem.hy300-remoto`. Repo público `puppe1990/hy300-remoto`. Vanilla `web/` (no TypeScript; public maps use JSDoc). Precisa do `adb` no PATH.

## Caveats

- Host padrão `192.168.15.4:3268`. Gravado em `host.json` no config dir do app.
- DNS sinkhole da LAN: `scripts/dnssink/` (`install.sh` / `update-hosts.sh` / `uninstall.sh`). Payload no HY300 em `/data/local/dnssink`. Não mexer em `daemon12138`. Testes do builder não falam com o projetor.
- Preview é `adb exec-out screencap` (raw RGBA, sem `-p`) a cada 500 ms, PNG no Mac. Um frame por vez; estoura em 4s. PNG no dispositivo (`-p`) leva 6–8s com vídeo ao vivo e o preview fica preto. `#preview-wrap` é `div` (não `<button>`). Toque no visor é `pointerdown` no wrap — recarregar `img.src` a cada frame cancela `click`. Mapeia com `object-fit: contain` (letterbox ignora) contra a caixa pintada (`tapTargetBox`). Janela 720×980.
- Roda no preview (e scroll de dois dedos) manda `input swipe` vertical: coalesce 50 ms, viagem 120–240 px, duração 200 ms. Letterbox ignora e zera o accum. `deltaX` fica de fora. Lock próprio, separado do screencap. Sem timeout extra no swipe.
- Modo Mouse (sessão, default off): botão **Mouse** no pad; visor deixa de mandar `input tap` e passa `input mouse motionevent MOVE|DOWN|UP` no mesmo `mapTap`. MOVE junta 50 ms e pula pixel repetido. Esc sai do modo e não manda BACK; Voltar do pad sempre BACK. Lock próprio, separado de screencap e swipe. Não grava o modo. Não mexe em `persist.sys.txcz.nfmouse` nem `daemon12138`.
- Trait `AdbDevice` / argv em `src-tauri/src/adb/mod.rs`; spawn+timeout em `adb/run.rs`. Testes não disparam o binário `adb` (só o caso do executável ausente e o timeout com um `sleep` fake).
- Feature `desktop` (default) liga Tauri. CI: `cargo test --lib --no-default-features`. `build.rs` chama `tauri_build` só com `CARGO_FEATURE_DESKTOP`. Doctest de `AdbDevice` não roda em `--lib`; cubra com `FakeAdb`.
- Mapas de tecla, tap, swipe, brilho, energia, lâmpada e mouse duplicados em Rust e `web/js/*.mjs` — goldens iguais. Do not merge them.
- Status `ligado` é lâmpada/usuário, não ADB. IR power deixa o SoC no ar com `LegacyGlobalActions`; o pad lê `dumpsys window displays` (`mCurrentFocus`) e mostra `desligado`. Tela off/on do pad escreve Lcd_en e Lcd_power (0/255) via `adb shell sh -c` — não é KEYCODE_POWER (isso abre o desligar do Android). Status combina GlobalActions **ou** GPIO 0 (`combinePower`). Não mexer em `daemon12138`.
- Brilho do pad: `persist.vendor.brightness.main` 0–100 passo 5 + `testgamma --brightness` (WB/gamma de `/system/bin/testgammatx.sh`). Não é keyevent. Não mexer em `daemon12138`.
- Ícone: `src-tauri/app-icon.png` → `./node_modules/.bin/tauri icon src-tauri/app-icon.png -o src-tauri/icons`. README mostra `docs/icon.png`. `tauri dev` não recarrega o Dock — reinicie depois de mudar o icns.
- Local: `npm run dev`. Gate: `npm run ci`.

## Code style

- Functions: 4-20 lines. Split if longer.
- Files: under 500 lines. Split by responsibility.
- One thing per function, one responsibility per module (SRP).
- Names: specific and unique. Avoid `data`, `handler`, `Manager`.
  Prefer names that return <5 grep hits in the codebase.
- Types: explicit. No `any`. JS public APIs get JSDoc; Rust public APIs get rustdoc.
- Dual key/tap/swipe/brightness/power/lamp/mouse maps are the allowed duplication. Extract anything else.
- Early returns over nested ifs. Max 2 levels of indentation for control flow.
- Exception messages must include the offending value and expected shape.

## Comments

- Keep intent/provenance comments. Don't strip them on refactor.
- Write WHY, not WHAT. Skip `// increment counter` above `i++`.
- Docstrings on public functions: intent + one usage example.
- Reference issue numbers / commit SHAs when a line exists because
  of a specific bug or upstream constraint.

## Tests

- Tests run with a single command: `npm test`
  (`node --test test/*.test.mjs` + `cargo test --manifest-path src-tauri/Cargo.toml --lib --no-default-features --quiet`).
- Every new function gets a test. Bug fixes get a regression test.
- Maps and host: `@faker-js/faker` (JS) and crate `fake` (Rust). Named `FakeAdb` for I/O. Do not talk to the projector.
- Tests must be F.I.R.S.T: fast, independent, repeatable,
  self-validating, timely.

## Dependencies

- Inject dependencies through constructor/parameter, not global/import.
- Wrap the `adb` binary behind `AdbDevice`. `CliAdb` is the CLI; tests use `FakeAdb`.
- Centralize config (hosts, model names, feature flags) — one source of truth. Default host lives in `host.rs` / `host.mjs`.

## Structure

- Follow Tauri 2: `src-tauri/` crate, `web/` frontend, `test/*.test.mjs`.
- `adb/mod.rs` trait+argv+CliAdb+FakeAdb; `adb/run.rs` spawn/timeout; `adb/raw.rs` Android 12 RGBA header → PNG; `keys.rs` / `tap.rs` / `host.rs` / `preview.rs` / `swipe.rs` / `brightness.rs` / `power.rs` / `lamp.rs` / `mouse.rs` duals of `web/js/*.mjs`; `main.rs` Tauri commands + `AdbSession`. Trait `AdbDevice` inclui `lamp_read`/`lamp_write` e `mouse`. Command `set_lamp` / `send_mouse`.
- Prefer small focused modules over god files.

## Formatting

- Prettier (`.prettierrc.json`), ESLint, `cargo fmt`. Don't discuss style.
- `npm run ci` = format:check + lint + test.

## Logging

- No observability logger.

## Defensive programming

- Timeout: `SCREENCAP_TIMEOUT` 4s on raw `screencap` only. Do not add retries, circuit breakers, or rate limits. Do not bring back `screencap -p`.
