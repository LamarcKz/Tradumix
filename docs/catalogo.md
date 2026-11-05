# Catálogo de traduções (`indice.json`)

O app lê um índice público com todas as traduções publicadas. Por padrão:

```
https://raw.githubusercontent.com/LamarcKz/Tradumix-catalogo/main/indice.json
```

O endereço pode ser trocado pela preferência `url_catalogo` no banco do app (`tradumix.db`, tabela
`preferencias`), na pasta de dados.

```json
{
  "formato": 1,
  "atualizado": "2026-11-05",
  "traducoes": [
    {
      "formato": 1,
      "id": "jak2-usa-ptbr",
      "jogo": { "id": "jak2", "nome": "Jak II", "console": "ps2" },
      "versao": "1.0",
      "estado": "beta",
      "idioma": "pt-BR",
      "data": "2026-11-05",
      "origem": { "serial": "SCUS-97265", "regiao": "usa", "hash_ra": "f51c94dd47cc2eb385d5733389428121", "tamanho": 4410671104 },
      "resultado": { "hash_ra": "f51c94dd47cc2eb385d5733389428121", "tamanho": 4410671104 },
      "tradutores": [ { "nome": "Nome do tradutor", "link": null } ],
      "arquivos": { "patch": "patch.xdelta" },
      "download": {
        "patch": { "url": "https://github.com/LamarcKz/Tradumix-catalogo/releases/download/jak2-usa-ptbr-v1.0/jak2-usa-ptbr-v1.0.xdelta", "sha256": "…64 hex…", "tamanho": 238940 }
      },
      "midia": { "trailer_youtube": "Pj-ueiUYEbg" }
    }
  ]
}
```

Cada item de `traducoes` é uma ficha no mesmo formato de [ficha.md](ficha.md), com o bloco
`download` obrigatório. Regras:

- `formato` do índice: hoje só `1`. Ids não podem se repetir.
- Todas as URLs são `https://`.
- `sha256` tem 64 caracteres hexadecimais e `tamanho` é o tamanho exato em bytes. O app confere
  os dois em todo download e descarta o arquivo que não bater.
- `arquivos` dá os nomes com que cada arquivo é guardado no cache do app.

## Capa e trailer: só referências

O catálogo não guarda imagens nem vídeos de terceiros.

- **Capa:** sem `capa` própria na ficha, o app busca pelo serial em
  `https://raw.githubusercontent.com/xlenore/ps2-covers/main/covers/default/<serial>.jpg`
  (o acervo que o PCSX2 usa) e guarda no cache. Só aceita JPEG ou PNG.
- **Fundo da página do jogo:** a capa ampliada e desfocada.
- **Trailer:** `midia.trailer_youtube` é o id do vídeo no YouTube (11 caracteres). O app o mostra no
  player do YouTube (`youtube-nocookie.com/embed`), sem som e em loop; dá para desligar em
  Configurações.

Os blocos `download.capa` e `download.fundo` continuam aceitos para imagens próprias (por exemplo,
arte feita pelo tradutor).

Uma tradução publicada preserva o hash do RetroAchievements: `origem.hash_ra` e
`resultado.hash_ra` são iguais.
