# Tradumix

App para Windows que aplica traduções de jogos de PS2 na ISO que você já tem.
Você escolhe o jogo, seleciona a sua ISO e recebe uma ISO traduzida ao lado da original.
Toda tradução do catálogo preserva o hash do RetroAchievements, então as conquistas
continuam funcionando.

O app nunca distribui o jogo: ele usa só o patch (`.xdelta`) da tradução.

> Em desenvolvimento. Nesta versão as traduções vêm de uma pasta local de rascunhos (modo de teste).

## Estrutura

| Pasta | O que é |
|---|---|
| `crates/nucleo` | Biblioteca Rust: leitura de ISO, identidade do jogo e hash do RetroAchievements, pacotes de tradução, aplicação do patch e histórico. |
| `src-tauri` | O app Tauri: expõe o núcleo como comandos para a interface. |
| `src` | A interface em React: início, página do jogo, progresso, pronto e configurações. |
| `scripts` | `baixar-xdelta3.mjs`: baixa o xdelta3 oficial e confere o SHA-256. |
| `docs/ficha.md` | Formato da ficha de tradução (`ficha.json`). |

## Como rodar

Requer Node.js 24 e Rust (edição 2024), no Windows.

```
npm install
npm run xdelta3
npm run dev
```

`npm run xdelta3` baixa o [xdelta3](https://github.com/jmacd/xdelta) 3.2.2 (Apache 2.0) e grava
em `src-tauri/binaries/`, onde o Tauri o leva junto com o app.

No app, em Configurações, escolha a pasta de rascunhos: cada tradução fica numa subpasta com o
`ficha.json` e o patch (ver [docs/ficha.md](docs/ficha.md)).

## Testes

```
npm test
```

Roda a checagem de tipos, os testes das telas (Vitest) e os testes do Rust. Os testes com um
jogo de verdade ficam de fora e rodam à parte, com as variáveis `TRADUMIX_PACOTES`,
`TRADUMIX_ISO_JAK2` e `TRADUMIX_SAIDA`:

```
cargo test -p nucleo --test jak2_real -- --ignored
```

## Licença

MIT. Veja [LICENSE](LICENSE). O xdelta3 é distribuído sob a licença Apache 2.0.
