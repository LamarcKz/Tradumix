# Catálogo de traduções (`indice-v2.json`)

O app lê um índice público com todas as traduções publicadas. Por padrão:

```
https://raw.githubusercontent.com/LamarcKz/Tradumix-catalogo/main/indice-v2.json
```

O endereço pode ser trocado pela preferência `url_catalogo` no banco do app (`tradumix.db`, tabela
`preferencias`), na pasta de dados.

```json
{
  "formato": 2,
  "atualizado": "2026-11-05",
  "traducoes": [
    {
      "formato": 2,
      "id": "jak2-usa-ptbr",
      "jogo": { "id": "jak2", "nome": "Jak II" },
      "plataforma": "ps2",
      "instalacao": {
        "tipo": "imagem-xdelta",
        "patch": {
          "nome": "patch.xdelta",
          "url": "https://github.com/LamarcKz/Tradumix-catalogo/releases/download/jak2-usa-ptbr-v1.0/jak2-usa-ptbr-v1.0.xdelta",
          "sha256": "…64 hex…",
          "tamanho": 238940
        }
      },
      "versao": "1.0",
      "estado": "beta",
      "idioma": "pt-BR",
      "data": "2026-11-05",
      "origem": { "serial": "SCUS-97265", "regiao": "usa", "hash_ra": "f51c94dd47cc2eb385d5733389428121", "tamanho": 4410671104 },
      "resultado": { "hash_ra": "f51c94dd47cc2eb385d5733389428121", "tamanho": 4410671104 },
      "tradutores": [ { "nome": "Nome do tradutor", "link": null } ],
      "midia": { "trailer_youtube": "Pj-ueiUYEbg" }
    }
  ]
}
```

Cada item de `traducoes` é uma ficha como em [ficha.md](ficha.md). Regras:

- O índice inteiro só é recusado se o JSON não abrir, faltar `traducoes` ou o `formato` não for `2`.
  Nesse caso o app continua com o último catálogo baixado.
- Cada ficha é lida sozinha. Fica de fora, sem derrubar as outras: plataforma ou tipo de instalação
  que esta versão do app não conhece, ficha que não passa nas regras de [ficha.md](ficha.md), URL que
  não é `https://` e `sha256` mal formado. Com `id` repetido, vale a primeira.
- Uma tradução que sai do índice (ou fica de fora) também sai do cache do app.

## Capa e trailer: só referências

O catálogo não guarda imagens nem vídeos de terceiros.

- **Capa:** sem `imagens.capa`, o app busca a capa no acervo da plataforma (no PS2, o
  `xlenore/ps2-covers`, pelo serial) e guarda no cache. Só aceita JPEG ou PNG.
- **Fundo da página do jogo:** a capa ampliada e desfocada.
- **Trailer:** `midia.trailer_youtube` é o id do vídeo no YouTube. O app o mostra no player do
  YouTube (`youtube-nocookie.com/embed`), sem som e em loop; dá para desligar em Configurações.

## Versões antigas do app

O `indice.json` (formato 1, só PS2) é o índice lido pelas versões do app até a 0.2. Ele continua no
catálogo para essas versões; as traduções novas entram no `indice-v2.json`.
