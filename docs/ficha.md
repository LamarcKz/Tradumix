# Ficha de tradução (formato 2)

Cada tradução do catálogo é descrita por uma ficha. O app guarda cada uma numa pasta do cache
(`ficha.json`, o patch baixado e as imagens).

```json
{
  "formato": 2,
  "id": "jak2-usa-ptbr",
  "jogo": { "id": "jak2", "nome": "Jak II" },
  "plataforma": "ps2",
  "instalacao": {
    "tipo": "imagem-xdelta",
    "patch": { "nome": "patch.xdelta", "url": "https://…", "sha256": "…64 hex…", "tamanho": 238940 }
  },
  "versao": "1.0",
  "estado": "beta",
  "idioma": "pt-BR",
  "data": "2026-11-05",
  "origem": { "serial": "SCUS-97265", "regiao": "usa", "hash_ra": "f51c94dd47cc2eb385d5733389428121", "tamanho": 4410671104 },
  "resultado": { "hash_ra": "f51c94dd47cc2eb385d5733389428121", "tamanho": 4410671104 },
  "tradutores": [ { "nome": "Nome do tradutor", "link": "https://…" } ],
  "permissao": null,
  "descricao": "…",
  "traduzido": ["Menus", "Missões", "Legendas das cenas"],
  "limitacoes": ["Algumas palavras desenhadas em textura continuam em inglês."],
  "imagens": { "capa": { "nome": "capa.jpg", "url": "https://…", "sha256": "…", "tamanho": 51234 } },
  "midia": { "trailer_youtube": "Pj-ueiUYEbg" }
}
```

| Campo | Obrigatório | Significado |
|---|---|---|
| `formato` | sim | Versão do formato da ficha: `2`. |
| `id` | sim | Identificador único da tradução. Vira nome de pasta: sem barras, `:` nem ponto no início. |
| `jogo.id`, `jogo.nome` | sim | Traduções com o mesmo `jogo.id` aparecem num card só, mesmo em plataformas diferentes. |
| `plataforma` | sim | Plataforma do jogo. Hoje o app conhece `ps2`. |
| `instalacao` | sim | Como a tradução é instalada (`tipo`) e o que esse tipo baixa. Hoje: `imagem-xdelta`. |
| `versao` | sim | Versão da tradução, comparada por partes numéricas (`1.9 < 1.10`). |
| `estado` | sim | `rascunho`, `beta` ou `publicada`. |
| `idioma` | sim | Idioma de destino (`pt-BR`). |
| `data` | sim | Data da versão, `AAAA-MM-DD`. |
| `origem`, `resultado` | sim | A cópia que a tradução espera e a que ela gera. O formato é da plataforma (abaixo). |
| `tradutores` | sim | Créditos, cada um com nome e link opcional. |
| `permissao` | não | Registro da permissão do autor, para traduções de terceiros. |
| `descricao`, `traduzido`, `limitacoes` | não | Textos da página do jogo. |
| `imagens.capa`, `imagens.fundo` | não | Imagens próprias da tradução. Sem capa própria, o app busca a do acervo da plataforma. |
| `midia.trailer_youtube` | não | Id do vídeo no YouTube (11 caracteres). |

Todo arquivo baixado (`instalacao.patch`, `imagens.*`) tem `nome` (só o nome do arquivo, sem pastas,
e diferente de `ficha.json`), `url` (`https://`), `sha256` (64 hex) e `tamanho` em bytes. O app
confere os dois em todo download e descarta o arquivo que não bater.

## Tipos de instalação

- **`imagem-xdelta`:** `patch` é um `.xdelta` aplicado na imagem do jogo da pessoa; a cópia traduzida
  é gravada ao lado da original, que não é alterada.

## Plataformas

### `ps2` (PlayStation 2)

- `origem`: `serial`, `regiao` (`usa`, `europa`, `japao`), `hash_ra` (32 hex) e `tamanho` da ISO.
- `resultado`: `hash_ra` e `tamanho` da ISO traduzida.
- O hash do RetroAchievements é o MD5 do nome do executável (como no `BOOT2` do `SYSTEM.CNF`, sem
  `cdrom0:\` e sem `;1`) seguido dos bytes do executável. Uma tradução publicada mantém
  `origem.hash_ra` igual a `resultado.hash_ra`.
- Capa sem `imagens.capa`: acervo `xlenore/ps2-covers`, pelo serial.

Uma ficha de plataforma ou tipo de instalação que o app não conhece fica de fora da lista do app.
