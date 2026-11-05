# Como contribuir

## Branches

- `main`: a versão estável. Só muda por Pull Request vindo da `desenvolvimento`, com os testes passando.
- `desenvolvimento`: o trabalho do dia a dia.
- Mudanças grandes ganham uma branch própria a partir da `desenvolvimento`.

## Commits

As mensagens seguem o [Conventional Commits](https://www.conventionalcommits.org/pt-br/), em português:

- `feat:` novidade para quem usa o app;
- `fix:` correção;
- `docs:`, `build:`, `ci:`, `test:`, `refactor:`, `chore:` para o resto.

O número da próxima versão sai disso (`npm run versao`): `feat` sobe o número do meio, só `fix` sobe o último.

## Antes de enviar

```
npm test
```

O repositório tem uma trava (`.githooks/pre-commit`) que barra commits com dados pessoais listados em
`.git/private-patterns.txt`, um arquivo que fica fora do Git. Para ativar: `git config core.hooksPath .githooks`.

## Traduções

As traduções ficam no [catálogo](https://github.com/LamarcKz/Tradumix-catalogo). Toda tradução precisa
preservar o hash do RetroAchievements e ter a permissão do autor.
