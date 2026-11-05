# Tradumix

App para Windows que aplica traduções de jogos de PS2 na ISO que você já tem.
Você escolhe o jogo, seleciona a sua ISO e recebe uma ISO traduzida ao lado da original.
Toda tradução do catálogo preserva o hash do RetroAchievements, então as conquistas
continuam funcionando.

O app nunca distribui o jogo: ele usa só o patch (`.xdelta`) da tradução.

> Em desenvolvimento.

## Baixar

Baixe a versão mais recente na [página de releases](https://github.com/LamarcKz/Tradumix/releases/latest):

- **Instalador** (`Tradumix_<versão>_x64-setup.exe`): instala só para o seu usuário, sem pedir permissão de administrador.
- **Portátil** (`Tradumix-<versão>-portable.zip`): extraia e abra o `Tradumix.exe`; os dados ficam na pasta `dados`, ao lado.

O Tradumix ainda não tem assinatura de código, então o Windows pode avisar que o app não é
conhecido ("O Windows protegeu o computador"). Para abrir mesmo assim, clique em
**Mais informações** e depois em **Executar assim mesmo**.

As traduções vêm do [catálogo público](https://github.com/LamarcKz/Tradumix-catalogo). O app avisa
quando sai uma versão nova, e você escolhe quando atualizar.

## Estrutura

| Pasta | O que é |
|---|---|
| `crates/nucleo` | Biblioteca Rust: leitura de ISO, identidade do jogo e hash do RetroAchievements, pacotes de tradução, aplicação do patch e histórico. |
| `src-tauri` | O app Tauri: expõe o núcleo como comandos para a interface. |
| `src` | A interface em React: início, página do jogo, progresso, pronto e configurações. |
| `scripts` | `baixar-xdelta3.mjs`: baixa o xdelta3 oficial e confere o SHA-256. |
| `docs/ficha.md` | Formato da ficha de tradução (`ficha.json`). |
| `docs/catalogo.md` | Formato do catálogo (`indice.json`) que o app baixa do GitHub. |

## Como rodar

Requer Node.js 24 e Rust (edição 2024), no Windows.

```
npm install
npm run xdelta3
npm run dev
```

`npm run xdelta3` baixa o [xdelta3](https://github.com/jmacd/xdelta) 3.2.2 (Apache 2.0) e grava
em `src-tauri/binaries/`, onde o Tauri o leva junto com o app.

## Catálogo

Ao abrir, o app baixa o catálogo público ([docs/catalogo.md](docs/catalogo.md)) e guarda uma cópia na
pasta de dados dele (`catalogo/`). O patch de cada tradução só é baixado na hora de traduzir, e todo
download é conferido pelo SHA-256 e pelo tamanho. Sem internet, o app mostra o último catálogo baixado
e deixa traduzir o que já tem o patch no PC.

Para testar com outro catálogo, defina `url_catalogo` no `config.json` da pasta de dados do app.

## Modo de teste (rascunhos)

Em Configurações, escolha uma pasta de rascunhos: cada tradução fica numa subpasta com o
`ficha.json` e o patch (ver [docs/ficha.md](docs/ficha.md)). Os rascunhos aparecem junto do catálogo,
com o selo "Rascunho".

## Testes

```
npm test
```

Roda a checagem de tipos, os testes das telas (Vitest) e os testes do Rust (inclusive os de rede,
com um servidor HTTP local). Os testes com um jogo ou um catálogo de verdade ficam de fora e rodam à
parte, com as variáveis `TRADUMIX_PACOTES`, `TRADUMIX_ISO_JAK2`, `TRADUMIX_SAIDA`,
`TRADUMIX_INDICE` e `TRADUMIX_CATALOGO`:

```
cargo test -p nucleo --test jak2_real --test catalogo -- --ignored
```

## Contribuir

Veja [CONTRIBUTING.md](CONTRIBUTING.md).

## Licença

MIT. Veja [LICENSE](LICENSE). O xdelta3 é distribuído sob a licença Apache 2.0.
