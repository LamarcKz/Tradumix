# Tradumix

App para Windows que aplica traduções de jogos de PS2 na ISO que você já tem.
Você escolhe o jogo, seleciona a sua ISO e recebe uma ISO traduzida ao lado da original.
Toda tradução do catálogo preserva o hash do RetroAchievements, então as conquistas
continuam funcionando.

O app nunca distribui o jogo: ele baixa só o patch (`.xdelta`) da tradução.

> Em desenvolvimento.

## Estrutura

| Pasta | O que é |
|---|---|
| `crates/nucleo` | Biblioteca Rust: leitura de ISO, identidade do jogo e hash do RetroAchievements, pacotes de tradução, aplicação do patch. |

## Licença

MIT. Veja [LICENSE](LICENSE).
