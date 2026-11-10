// Espelho dos tipos que os comandos do app (src-tauri/src/comandos.rs) devolvem.

export type EstadoTraducao = "rascunho" | "beta" | "publicada"
export type Etapa = "conferindo" | "baixando" | "aplicando" | "verificando"

export interface Tradutor {
  nome: string
  link: string | null
}

/** O que a interface sabe de uma plataforma (vem do núcleo). */
export interface InfoPlataforma {
  id: string
  /** Nome para mostrar ("PlayStation 2"). */
  nome: string
  /** Nome curto, no card e no filtro ("PS2"). */
  sigla: string
  /** Como se chama o arquivo do jogo que a pessoa escolhe ("ISO"). */
  arquivo: string
  /** Extensões aceitas no seletor de arquivo. */
  extensoes: string[]
  /** A plataforma tem RetroAchievements. */
  tem_ra: boolean
  /** A pessoa escolhe a pasta do jogo instalado (PC) em vez de um arquivo. */
  escolhe_pasta: boolean
}

export interface Traducao {
  id: string
  versao: string
  estado: EstadoTraducao
  idioma: string
  /** Para qual versão do jogo é a tradução ("versão USA (SCUS-97265)"). */
  detalhes: string
  /** O que a pessoa precisa escolher para traduzir. */
  instrucao: string
  tradutores: Tradutor[]
  permissao: string | null
  descricao: string
  traduzido: string[]
  limitacoes: string[]
  data: string
  fundo: string | null
  novo: boolean
  patch_disponivel: boolean
  tamanho_download: number | null
  trailer_youtube: string | null
  /** Selo do RetroAchievements; `null` em plataforma sem RetroAchievements. */
  ra: SituacaoRa | null
  /** A tradução põe na pasta do jogo um arquivo que roda (carregador de mods, por exemplo). */
  contem_programa: boolean
}

export type SituacaoRa = "compativel" | "nao_reconhece"

/** As traduções de um jogo numa plataforma. */
export interface PlataformaDoJogo {
  info: InfoPlataforma
  capa: string | null
  traducoes: Traducao[]
}

/** Um card: o jogo com as plataformas em que há tradução. */
export interface Jogo {
  id: string
  nome: string
  plataformas: PlataformaDoJogo[]
}

export interface Atualizacao {
  pacote_id: string
  versao_gerada: string
  versao_nova: string
}

export interface ListaJogos {
  jogos: Jogo[]
  atualizacoes: Atualizacao[]
  catalogo: SituacaoCatalogo
  mostrar_trailers: boolean
}

export interface SituacaoCatalogo {
  offline: boolean
  atualizado_em: string | null
  aviso: string | null
}

export interface Conferencia {
  serve: boolean
  mensagem: string | null
  /** Serve, mas com um aviso (PC com outra versão do jogo). */
  aviso: string | null
  saida_sugerida: string
  ja_existe: boolean
}

export interface Resultado {
  saida: string
  /** `null` em plataforma sem RetroAchievements. */
  hash_ra: string | null
  compativel_ra: boolean | null
  segundos: number
}

/** O que "Remover tradução" fez na pasta do jogo. */
export interface Remocao {
  removidos: number
  restaurados: number
  /** Arquivos que a loja atualizou depois da instalação e ficaram como estão. */
  mantidos: string[]
}

export interface Progresso {
  etapa: Etapa
  fracao: number
}

/** Versão nova do Tradumix encontrada no GitHub. `portatil`: o app aberto é a versão portátil. */
export interface VersaoNova {
  versao: string
  notas: string
  portatil: boolean
}

/** A versão do app aberto e se é a portátil. */
export interface VersaoDoApp {
  versao: string
  portatil: boolean
}
