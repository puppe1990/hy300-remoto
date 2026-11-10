# HY300 Remoto

Tauri 2 no Mac: preview + pad + teclado do projetor HY300 via ADB wireless. Identifier `com.gestaobem.hy300-remoto`. Repo público `puppe1990/hy300-remoto`. Vanilla `web/` (no TypeScript; public maps use JSDoc). Precisa do `adb` no PATH.

## Caveats

- Host padrão `192.168.15.4:3268`. Gravado em `host.json` no config dir do app.
- DNS sinkhole da LAN: `scripts/dnssink/` (`install.sh` / `update-hosts.sh` / `uninstall.sh`). Payload no HY300 em `/data/local/dnssink`. Não mexer em `daemon12138`. Testes do builder não falam com o projetor.
- Preview é `screencap -p` a cada 500 ms, não scrcpy. Um frame por vez; `screencap` estoura em 4s (vídeo ~3,5s). Clique mapeia com `object-fit: contain` (letterbox ignora).
- Trait `AdbDevice` / argv em `src-tauri/src/adb/mod.rs`; spawn+timeout em `adb/run.rs`. Testes não disparam o binário `adb` (só o caso do executável ausente e o timeout com um `sleep` fake).
- Feature `desktop` (default) liga Tauri. CI: `cargo test --lib --no-default-features`. `build.rs` chama `tauri_build` só com `CARGO_FEATURE_DESKTOP`. Doctest de `AdbDevice` não roda em `--lib`; cubra com `FakeAdb`.
- Mapas de tecla e tap duplicados em Rust e `web/js/*.mjs` — goldens iguais. Do not merge them.
- Ícone: `src-tauri/app-icon.png` → `./node_modules/.bin/tauri icon src-tauri/app-icon.png -o src-tauri/icons`. README mostra `docs/icon.png`. `tauri dev` não recarrega o Dock — reinicie depois de mudar o icns.
- Local: `npm run dev`. Gate: `npm run ci`.

## Code style

- Functions: 4-20 lines. Split if longer.
- Files: under 500 lines. Split by responsibility.
- One thing per function, one responsibility per module (SRP).
- Names: specific and unique. Avoid `data`, `handler`, `Manager`.
  Prefer names that return <5 grep hits in the codebase.
- Types: explicit. No `any`. JS public APIs get JSDoc; Rust public APIs get rustdoc.
- Dual key/tap maps are the allowed duplication. Extract anything else.
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
- `adb/mod.rs` trait+argv+CliAdb+FakeAdb; `adb/run.rs` spawn/timeout; `keys.rs` / `tap.rs` / `host.rs` / `preview.rs` duals of `web/js/*.mjs`; `main.rs` Tauri commands + `AdbSession`.
- Prefer small focused modules over god files.

## Formatting

- Prettier (`.prettierrc.json`), ESLint, `cargo fmt`. Don't discuss style.
- `npm run ci` = format:check + lint + test.

## Logging

- No observability logger.

## Defensive programming

- Timeout: `SCREENCAP_TIMEOUT` 4s on `screencap -p` only. Do not add retries, circuit breakers, or rate limits.
