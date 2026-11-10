# HY300 Remoto

Desktop Tauri 2 no Mac que controla o projetor HY300 na LAN via ADB wireless. Preview da tela + pad + teclado.

Data: 2026-11-10  
Pasta: `produtos/hy300-remoto`

## Objetivo

Do Mac, na mesma Wi-Fi, ver a tela do HY300 (~1–2 fps) e mandar D-pad / OK / voltar / home / volume / power, além de toque no preview. Um aparelho só. Testes não falam com o projetor.

## Fora do v1

- Descoberta mDNS / lista de ADB na LAN
- Swipe / arraste
- scrcpy / H.264
- Mandar mídia (DLNA / AirPlay)
- Protocolo Android TV Remote (PIN)
- ADB em Rust puro (sem o CLI `adb`)
- Root, instalar APK, shell livre

## Stack

Cópia do ram-bar / hid-tester:

- Tauri 2, `web/` vanilla (sem TypeScript, sem React)
- Lógica em `src-tauri/src/*.rs` atrás da feature `desktop` (default). Ubuntu CI: `cargo test --lib --no-default-features`
- TDD: `node --test test/*.test.mjs` + `@faker-js/faker`; `cargo test --lib --no-default-features` + crate `fake`
- `npm test` = os dois. `npm run ci` = format:check + lint + test
- UI em português. Identifiers em inglês (`up`, `host`, `keyevent`)
- Bundle `com.gestaobem.hy300-remoto`. Precisa do `adb` no PATH (já está neste Mac)

## Host

- Padrão: `192.168.15.4:3268` (serial ADB wireless do HY300)
- Gravado em JSON `{ "host": "…" }` no diretório de config do app
- Campo na UI para mudar; valor inválido não grava
- Válido: `host:porta` com porta 1–65535, host não vazio (IPv4, hostname, `.local`)
- Erro cita o valor recebido e a forma esperada (`host:porta`)
- Ao abrir: `adb connect {host}` e reconecta se o status cair
- Sem mDNS

## Teclas

Android `input keyevent` (códigos AOSP):

| Pad / teclado Mac             | Código | KeyEvent    |
| ----------------------------- | ------ | ----------- |
| ▲ / ArrowUp                   | 19     | DPAD_UP     |
| ▼ / ArrowDown                 | 20     | DPAD_DOWN   |
| ◀ / ArrowLeft                 | 21     | DPAD_LEFT   |
| ▶ / ArrowRight                | 22     | DPAD_RIGHT  |
| OK / Enter, NumpadEnter       | 23     | DPAD_CENTER |
| Voltar / Escape               | 4      | BACK        |
| Home / KeyH                   | 3      | HOME        |
| Vol + / Equal, NumpadAdd      | 24     | VOLUME_UP   |
| Vol − / Minus, NumpadSubtract | 25     | VOLUME_DOWN |
| Power                         | 26     | POWER       |

Outro `event.code` → ignora (não manda keyevent). Mapas duplicados em Rust e `web/js/keys.mjs` com os mesmos goldens.

## Preview

- `adb -s {host} exec-out screencap -p` a cada 500 ms (2 fps)
- PNG vira `data:image/png;base64,…` no `<img>`
- Falha de um frame: mantém o último, status vira erro; o loop continua
- Sem preview contínuo tipo scrcpy

## Toque

Clique no preview mapeia para `input tap X Y` na resolução do frame (fallback 1280×720 se o PNG ainda não chegou).

O `<img>` usa `object-fit: contain`. Clique na faixa (letterbox) não manda tap.

Fórmula: escala = min(elemW/screenW, elemH/screenH); conteúdo centrado; coordenadas no conteúdo × 1/escala; arredonda e satura em `[0, screenW-1]` × `[0, screenH-1]`.

Sem swipe.

## ADB

Trait `AdbDevice`: `connect`, `keyevent`, `tap`, `screencap_png`. Implementação real monta argv do CLI `adb` e executa. Testes usam fake + funções puras de argv (não disparam o binário).

Comandos:

- `adb connect {host}`
- `adb -s {host} shell input keyevent {code}`
- `adb -s {host} shell input tap {x} {y}`
- `adb -s {host} exec-out screencap -p`

Erro inclui o argv e o que o processo devolveu.

## UI

Janela empilhada (~420×720, redimensionável):

1. Header: marca HY300 + status (`conectando…` / `ligado` / `sem conexão` / `falhou: …`)
2. Preview 16:9, clique = toque
3. D-pad + OK
4. Voltar, Home, Power
5. Vol − / Vol +
6. Campo host + botão Salvar
7. Dica: setas · Enter · Esc · H · +/−

Pele do mockup: fundo `#141110`, pad `#2a2620`, OK `#c45c26`, texto `#f4efe6`. Fonte do sistema. Sem Google Fonts CDN.

Foco na janela: teclado do Mac dispara o mapa. Campo host focado: teclado não manda keyevent (escreve o host).

## Testes

Um comando: `npm test`.

Cobrir: mapa de teclas (pad + `event.code`), host válido/inválido com faker, persistência em arquivo temp, mapeamento de tap (centro, canto, letterbox, saturacão), argv do adb, intervalo 500 ms.

Proibido: `adb` de verdade nos testes da CI.

## Arquivos

```
src-tauri/src/keys.rs      mapa pad/teclado → keycode
src-tauri/src/tap.rs       clique → (x, y)
src-tauri/src/host.rs      default, parse, persist
src-tauri/src/adb.rs       trait + argv + CLI
src-tauri/src/preview.rs   intervalo, data URL
src-tauri/src/lib.rs
src-tauri/src/main.rs      commands Tauri (feature desktop)
web/js/keys.mjs            dual do mapa
web/js/tap.mjs             dual do tap
web/js/host.mjs            dual do parse
web/js/app.js              invoke
web/index.html
web/css/remote.css
test/*.test.mjs
```
