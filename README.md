# Tradumix

App para Windows que aplica traduções de jogos na cópia que você já tem. Hoje o app instala
traduções de PS2: você escolhe o jogo, seleciona a sua ISO e recebe uma ISO traduzida ao lado da
original. Nos jogos com RetroAchievements, a tradução preserva o hash, então as conquistas
continuam funcionando.

As traduções vêm só do catálogo público. Cada tradução diz a plataforma do jogo; o app mostra só as
das plataformas que ele sabe instalar.

O app nunca distribui o jogo: ele usa só o patch (`.xdelta`) da tradução.

> Em desenvolvimento.

## Baixar

Baixe a versão mais recente na [página de releases](https://github.com/LamarcKz/Tradumix/releases/latest):

- **Instalador** (`Tradumix-Setup-<versão>.exe`): instala só para o seu usuário, sem pedir permissão de administrador.
- **Portátil** (`Tradumix-<versão>-portable.zip`): extraia e abra o `Tradumix.exe`; os dados ficam na pasta `dados`, ao lado.

O Tradumix ainda não tem assinatura de código, então o Windows pode avisar que o app não é
conhecido ("O Windows protegeu o computador"). Para abrir mesmo assim, clique em
**Mais informações** e depois em **Executar assim mesmo**.

As traduções vêm do [catálogo público](https://github.com/LamarcKz/Tradumix-catalogo). O app avisa
quando sai uma versão nova, e você escolhe quando atualizar.

## Estrutura

| Pasta | O que é |
|---|---|
| `crates/nucleo` | Biblioteca Rust: fichas e catálogo, plataformas (`plataformas/`: como reconhecer a cópia de cada uma), leitura de ISO e hash do RetroAchievements, aplicação do patch e histórico. |
| `src-tauri` | O app Tauri: expõe o núcleo como comandos para a interface. |
| `src` | A interface em React: início, página do jogo, progresso, pronto e configurações. |
| `scripts` | `baixar-xdelta3.mjs`: baixa o xdelta3 oficial e confere o SHA-256. |
| `docs/ficha.md` | Formato da ficha de tradução (`ficha.json`). |
| `docs/catalogo.md` | Formato do catálogo (`indice-v2.json`) que o app baixa do GitHub. |

## Como rodar

Requer Node.js 24 e Rust (edição 2024), no Windows.

```
npm install
npm run xdelta3
npm run dev
```

`npm run xdelta3` baixa o [xdelta3](https://github.com/jmacd/xdelta) 3.2.2 (Apache 2.0), confere o
SHA-256 e grava em `src-tauri/binaries/`, onde o Tauri o leva junto com o app. Ele vem da release
`xdelta3-v3.2.2` deste repositório, compilada do código oficial pelo workflow
[`xdelta3.yml`](.github/workflows/xdelta3.yml) com as opções da release oficial e a biblioteca do C
dentro do `.exe`, para não depender do `VCRUNTIME140.dll`.

O `npm run dev` mostra a marca **DEV** embaixo do logo e usa uma cópia dos dados do app instalado
(`%APPDATA%\io.github.tradumix Dev`); `npm run dev:nova-copia` refaz essa cópia.

## Lançar uma versão

- `npm run versao`: diz qual é a próxima versão pelos commits desde a última tag (`feat` sobe o número
  do meio, `fix` o último) e monta um rascunho das notas.
- `npm run dist`: roda os testes e gera em `dist-release/` o instalador, o `.zip` portátil e o
  `latest.json` assinado, que o app lê para avisar da versão nova. Precisa da chave das atualizações
  (`TAURI_SIGNING_PRIVATE_KEY` ou `.tauri/tradumix.key` na pasta do usuário), que nunca vai para o
  repositório. O build tira do `.exe` os caminhos do PC de quem compila.

## Catálogo

Ao abrir, o app baixa o catálogo público ([docs/catalogo.md](docs/catalogo.md)) e guarda uma cópia na
pasta de dados dele (`catalogo/`). O patch de cada tradução só é baixado na hora de traduzir, e todo
download é conferido pelo SHA-256 e pelo tamanho. Sem internet, o app mostra o último catálogo baixado
e deixa traduzir o que já tem o patch no PC.

Para testar com outro catálogo, defina a preferência `url_catalogo` no banco do app (`tradumix.db`,
tabela `preferencias`, na pasta de dados).

## Testes

```
npm test
```

Roda a checagem de tipos, os testes das telas (Vitest) e os testes do Rust (inclusive os de rede,
com um servidor HTTP local). Os testes com um jogo ou um catálogo de verdade ficam de fora e rodam à
parte, com as variáveis `TRADUMIX_PATCH_JAK2`, `TRADUMIX_ISO_JAK2`, `TRADUMIX_SAIDA`,
`TRADUMIX_INDICE` e `TRADUMIX_CATALOGO`:

```
cargo test -p nucleo --test jak2_real --test catalogo -- --ignored
```

## Contribuir

Veja [CONTRIBUTING.md](CONTRIBUTING.md).

## Licença

MIT. Veja [LICENSE](LICENSE). O xdelta3 é distribuído sob a licença Apache 2.0.
