# HY300 Remoto

Tauri 2 no Mac: preview + pad + teclado do projetor HY300 via ADB wireless.

Identifier `com.gestaobem.hy300-remoto`. Repo público `puppe1990/hy300-remoto`. Precisa do `adb` no PATH.

Ícone: `src-tauri/app-icon.png` → `npx tauri icon src-tauri/app-icon.png -o src-tauri/icons`.

## Caveats

- Host padrão `192.168.15.4:3268`. Gravado em `host.json` no config dir do app.
- Preview é `screencap -p` a cada 500 ms, não scrcpy. Um frame por vez; `screencap` estoura em 4s (vídeo ~3,5s).
- Clique no preview mapeia com `object-fit: contain` (letterbox ignora).
- Trait `AdbDevice` / argv em `adb.rs`. Testes não disparam o binário `adb` (só o caso do executável ausente).
- Feature `desktop` (default) liga Tauri. CI: `cargo test --lib --no-default-features`.
- Mapas de tecla e tap duplicados em Rust e `web/js/*.mjs` — goldens iguais.

## Tests

`npm test` = `node --test test/*.test.mjs` + `cargo test --lib --no-default-features`.
`npm run ci` = format:check + lint + test.
`@faker-js/faker` e crate `fake`. Sem ADB real na CI.

## Run

`npm run dev` a partir desta pasta.
