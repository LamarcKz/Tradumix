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
| `plataforma` | sim | Plataforma do jogo: `ps2`, `ps1` ou `pc`. |
| `instalacao` | sim | Como a tradução é instalada (`tipo`) e o que esse tipo baixa: `imagem-xdelta` (PS2 e PS1) ou `pasta-do-jogo` (PC). |
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

Todo arquivo baixado (`instalacao.patch` ou `instalacao.pacote`, `imagens.*`) tem `nome` (só o nome do arquivo, sem pastas,
e diferente de `ficha.json`), `url` (`https://`), `sha256` (64 hex) e `tamanho` em bytes. O app
confere os dois em todo download e descarta o arquivo que não bater.

## Tipos de instalação

- **`imagem-xdelta`:** `patch` é um `.xdelta` aplicado na imagem do jogo da pessoa; a cópia traduzida
  é gravada ao lado da original, que não é alterada. Vale para `ps2` e `ps1`.
- **`pasta-do-jogo`:** só para `pc`. `pacote` é um `.zip` com os arquivos da tradução e `arquivos`
  lista cada um: `caminho` (relativo à pasta do jogo, com `/`), `sha256` e `tamanho`. O `.zip` tem de
  ter exatamente esses arquivos, com esses SHA-256.

  ```json
  "instalacao": {
    "tipo": "pasta-do-jogo",
    "pacote": { "nome": "pacote.zip", "url": "https://…", "sha256": "…", "tamanho": 81234 },
    "arquivos": [ { "caminho": "mods/textos.sub", "sha256": "…", "tamanho": 51200 } ]
  }
  ```

  Regras de `caminho` (em `arquivos` e em `origem.referencias`): relativo, separado por `/`, sem `..`
  nem `.`, sem unidade (`C:`), sem começar com `/` ou `\`, sem pasta de nome vazio, sem `\ : * ? " < > |`,
  sem nome terminado em ponto ou espaço e sem nome reservado do Windows (`CON`, `PRN`, `AUX`, `NUL`,
  `COM1`–`COM9`, `LPT1`–`LPT9`, com ou sem extensão). Uma ficha que quebra uma regra fica de fora.

  Na instalação, o app extrai e confere o `.zip` inteiro antes de mexer na pasta do jogo. Os arquivos
  do jogo que forem trocados são guardados em `.tradumix/<id>/originais/`, dentro da pasta do jogo,
  para **Remover tradução** devolvê-los. Um `.exe` ou `.dll` que já existe na pasta do jogo nunca é
  substituído: a instalação é recusada. Se a loja atualizar um arquivo depois, a remoção deixa o
  arquivo da loja como está.

  **Contém programa:** a tradução mostra esse selo quando algum arquivo termina em `.exe`, `.dll`,
  `.bat`, `.cmd`, `.ps1`, `.vbs`, `.js`, `.py`, `.msi`, `.scr` ou `.com` (por exemplo, um carregador
  de mods).

## Plataformas

### `ps2` (PlayStation 2)

- `origem`: `serial`, `regiao` (`usa`, `europa`, `japao`), `hash_ra` (32 hex) e `tamanho` da ISO.
- `resultado`: `hash_ra` e `tamanho` da ISO traduzida.
- O hash do RetroAchievements é o MD5 do nome do executável (como no `BOOT2` do `SYSTEM.CNF`, sem
  `cdrom0:\` e sem `;1`) seguido dos bytes do executável. Uma tradução publicada mantém
  `origem.hash_ra` igual a `resultado.hash_ra`.
- Capa sem `imagens.capa`: acervo `xlenore/ps2-covers`, pelo serial.

### `ps1` (PlayStation)

- A pessoa escolhe o `.cue` (padrão Redump, um `.bin` por faixa). `.img`, `.pbp` e CHD não são aceitos.
- `origem`: `serial`, `regiao`, `hash_ra` (32 hex) e `tamanho` do `.bin` da faixa de dados.
- `resultado`: `hash_ra` e `tamanho` do `.bin` de dados traduzido.
- O patch vale para o `.bin` de dados. As faixas de áudio são copiadas e um `.cue` novo é gravado ao
  lado, com os nomes novos (`<nome> (Faixa N).bin` quando há mais de uma faixa).
- O hash do RetroAchievements segue o rcheevos: MD5 do caminho do executável como está na linha
  `BOOT` do `SYSTEM.CNF` (sem `cdrom:`, sem as barras do início e sem `;1`; ou `PSX.EXE`) seguido de
  `t_size + 2048` bytes do executável `PS-X EXE`.
- Capa sem `imagens.capa`: acervo `xlenore/psx-covers`, pelo serial. O card usa a caixa quadrada do
  estojo de CD (no PS2, a de DVD; no PC, a proporção 2:3 da Steam), para a capa aparecer inteira.

### `pc`

- A pessoa escolhe a pasta do jogo instalado. Sem RetroAchievements.
- `origem`: `loja` ("Steam", "GOG"…), `versao` (a versão do jogo testada) e `referencias`, uma lista
  de arquivos da pasta do jogo (`caminho`, `sha256`) que identificam essa versão. Opcional:
  `steam_app_id`, o id do jogo na Steam (o número no endereço da loja), que dá a capa do card.
- `resultado`: `{}`.
- Nenhuma referência na pasta: o app diz que a pasta não é do jogo. Alguma diferente ou faltando: o
  app instala com o aviso de que a versão do jogo é diferente da testada.
- Capa sem `imagens.capa`: com `steam_app_id`, a capa vertical da biblioteca da Steam
  (`library_600x900.jpg`); sem ele, o card fica sem capa.

## Selos

- **Compatível com RetroAchievements:** `origem.hash_ra` igual a `resultado.hash_ra`.
- **O RetroAchievements não reconhece esta cópia:** os dois diferentes.
- **Contém programa:** ver `pasta-do-jogo`.

Uma ficha de plataforma ou tipo de instalação que o app não conhece fica de fora da lista do app.
