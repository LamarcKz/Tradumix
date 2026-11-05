// Espelho dos tipos que os comandos do app (src-tauri/src/comandos.rs) devolvem.

export type EstadoTraducao = "rascunho" | "beta" | "publicada"
export type Etapa = "conferindo" | "baixando" | "aplicando" | "verificando"

export interface Tradutor {
  nome: string
  link: string | null
}

export interface Traducao {
  id: string
  versao: string
  estado: EstadoTraducao
  idioma: string
  regiao: string
  serial: string
  tradutores: Tradutor[]
  permissao: string | null
  descricao: string
  traduzido: string[]
  limitacoes: string[]
  data: string
  fundo: string | null
  origem: "catalogo" | "rascunho"
  novo: boolean
  patch_disponivel: boolean
  tamanho_download: number | null
  trailer_youtube: string | null
}

export interface Jogo {
  id: string
  nome: string
  console: string
  capa: string | null
  traducoes: Traducao[]
}

export interface PacoteInvalido {
  pasta: string
  motivo: string
}

export interface Atualizacao {
  pacote_id: string
  versao_gerada: string
  versao_nova: string
}

export interface ListaJogos {
  jogos: Jogo[]
  invalidos: PacoteInvalido[]
  atualizacoes: Atualizacao[]
  pasta_rascunhos: string | null
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
  saida_sugerida: string
  ja_existe: boolean
}

export interface Resultado {
  saida: string
  hash_ra: string
  compativel_ra: boolean
  segundos: number
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
