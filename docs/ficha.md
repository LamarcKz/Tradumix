# Ficha de tradução (`ficha.json`)

Cada pacote de tradução é uma pasta com um `ficha.json`, o patch `.xdelta` e, opcionalmente,
uma capa e um fundo. Todos os arquivos ficam na própria pasta do pacote: os nomes em
`arquivos` não podem ter barras, `:` nem `..`.

```json
{
  "formato": 1,
  "id": "jak2-usa-ptbr",
  "jogo": { "id": "jak2", "nome": "Jak II", "console": "ps2" },
  "versao": "1.0",
  "estado": "rascunho",
  "idioma": "pt-BR",
  "data": "2026-11-05",
  "origem": { "serial": "SCUS-97265", "regiao": "usa", "hash_ra": "f51c94dd47cc2eb385d5733389428121", "tamanho": 4410671104 },
  "resultado": { "hash_ra": "f51c94dd47cc2eb385d5733389428121", "tamanho": 4410671104 },
  "tradutores": [ { "nome": "Nome do tradutor", "link": "https://…" } ],
  "permissao": null,
  "descricao": "…",
  "traduzido": ["Menus", "Missões", "Legendas das cenas"],
  "limitacoes": ["Algumas palavras desenhadas em textura continuam em inglês."],
  "arquivos": { "patch": "patch.xdelta", "capa": "capa.jpg", "fundo": "fundo.jpg" }
}
```

| Campo | Obrigatório | Significado |
|---|---|---|
| `formato` | sim | Versão do formato da ficha. Hoje só `1`. |
| `id` | sim | Identificador único da tradução. |
| `jogo.id`, `jogo.nome`, `jogo.console` | sim | Agrupa traduções do mesmo jogo num card só. `console`: `ps2`. |
| `versao` | sim | Versão da tradução, comparada por partes numéricas (`1.9 < 1.10`). |
| `estado` | sim | `rascunho`, `beta` ou `publicada`. |
| `idioma` | sim | Idioma de destino (`pt-BR`). Vai no nome da ISO gerada (`PT-BR`). |
| `data` | sim | Data da versão, `AAAA-MM-DD`. |
| `origem` | sim | ISO que a tradução espera: serial, região (`usa`, `europa`, `japao`), hash do RetroAchievements e tamanho em bytes. |
| `resultado` | sim | Hash do RetroAchievements e tamanho da ISO traduzida. |
| `tradutores` | sim | Créditos, cada um com nome e link opcional. |
| `permissao` | não | Registro da permissão do autor, para traduções de terceiros. |
| `descricao`, `traduzido`, `limitacoes` | não | Textos da página do jogo. |
| `arquivos.patch` | sim | Nome do patch `.xdelta` na pasta do pacote. |
| `arquivos.capa`, `arquivos.fundo` | não | Imagens da grade e da página do jogo. |

Os dois hashes têm 32 caracteres hexadecimais. O hash do RetroAchievements de um jogo de
PS2 é o MD5 do nome do executável (como no `BOOT2` do `SYSTEM.CNF`, sem `cdrom0:\` e sem
`;1`) seguido dos bytes do executável.
