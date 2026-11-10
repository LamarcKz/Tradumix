// Dados de exemplo dos testes da interface, no formato que os comandos do app devolvem.

import type { InfoPlataforma, Jogo, ListaJogos, PlataformaDoJogo, Traducao } from "../tipos"

export const infoPs2: InfoPlataforma = {
  id: "ps2",
  nome: "PlayStation 2",
  sigla: "PS2",
  arquivo: "ISO",
  extensoes: ["iso"],
  tem_ra: true,
  escolhe_pasta: false,
  proporcao_capa: "7/10",
}

export const infoPs1: InfoPlataforma = {
  id: "ps1",
  nome: "PlayStation",
  sigla: "PS1",
  arquivo: "imagem (.cue)",
  extensoes: ["cue"],
  tem_ra: true,
  escolhe_pasta: false,
  proporcao_capa: "1/1",
}

export const infoPc: InfoPlataforma = {
  id: "pc",
  nome: "PC",
  sigla: "PC",
  arquivo: "pasta do jogo",
  extensoes: [],
  tem_ra: false,
  escolhe_pasta: true,
  proporcao_capa: "2/3",
}

export const traducaoJak2: Traducao = {
  id: "jak2-usa-ptbr",
  versao: "1.0",
  estado: "rascunho",
  idioma: "pt-BR",
  detalhes: "versão USA (SCUS-97265)",
  instrucao: "Use a sua ISO original da versão USA. Ela não é alterada: a traduzida é gravada ao lado.",
  tradutores: [{ nome: "Fulano de Tal", link: "https://exemplo.org" }],
  permissao: null,
  descricao: "Menus, missões e legendas em português.",
  traduzido: ["Menus", "Legendas das cenas"],
  limitacoes: ["Algumas texturas continuam em inglês."],
  data: "2026-11-05",
  fundo: null,
  novo: false,
  patch_disponivel: true,
  tamanho_download: null,
  trailer_youtube: null,
  ra: "compativel",
  contem_programa: false,
}

export function plataforma(info: InfoPlataforma, traducoes: Traducao[], capa: string | null = null): PlataformaDoJogo {
  return { info, capa, traducoes }
}

/** Jogo com uma plataforma (PS2) e uma tradução; `extra` muda campos da tradução. */
export function jogoPs2(extra: Partial<Traducao> = {}, base: Partial<Jogo> = {}, capa: string | null = null): Jogo {
  return { id: "jak2", nome: "Jak II", plataformas: [plataforma(infoPs2, [{ ...traducaoJak2, ...extra }], capa)], ...base }
}

/** Jak II no PS2 e no PC, cada um com a sua tradução. */
export function jogoPs2ePc(): Jogo {
  const pc: Traducao = {
    ...traducaoJak2,
    id: "jak2-pc-ptbr",
    versao: "2.0",
    detalhes: "versão Steam",
    instrucao: "Escolha a pasta do jogo.",
    ra: null,
  }
  return { id: "jak2", nome: "Jak II", plataformas: [plataforma(infoPs2, [traducaoJak2]), plataforma(infoPc, [pc])] }
}

export function listaDe(jogos: Jogo[], parcial: Partial<ListaJogos> = {}): ListaJogos {
  return { jogos, atualizacoes: [], catalogo: { offline: false, atualizado_em: "2026-11-05", aviso: null }, mostrar_trailers: true, ...parcial }
}
