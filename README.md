# HY300 Remoto

<p align="center">
  <img src="docs/icon.png" width="192" height="192" alt="HY300 Remoto">
</p>

Controle do projetor HY300 no Mac: preview da tela (2 fps), D-pad, teclado e toque via ADB wireless.

[![CI](https://github.com/puppe1990/hy300-remoto/actions/workflows/ci.yml/badge.svg)](https://github.com/puppe1990/hy300-remoto/actions/workflows/ci.yml)

```bash
cd produtos/hy300-remoto
npm install
npm test
npm run ci
npm run dev
```

`adb` precisa estar no PATH. Host padrão `192.168.15.4:3268`. Spec: `docs/superpowers/specs/2026-11-10-hy300-remoto-design.md`.
