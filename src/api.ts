// A única ponte entre as telas e o app em Rust. Os testes trocam este módulo por um falso.
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { open } from "@tauri-apps/plugin-dialog"

import type { Conferencia, ListaJogos, Progresso, Resultado, VersaoNova } from "./tipos"

export function listarJogos(): Promise<ListaJogos> {
  return invoke("listar_jogos")
}

/** Baixa o catálogo do GitHub (ou fica com o do cache, sem internet) e devolve a lista. */
export function sincronizarCatalogo(): Promise<ListaJogos> {
  return invoke("sincronizar_catalogo")
}

export function definirMostrarTrailers(mostrar: boolean): Promise<ListaJogos> {
  return invoke("definir_mostrar_trailers", { mostrar })
}

export function escolherPastaRascunhos(caminho: string): Promise<ListaJogos> {
  return invoke("escolher_pasta_rascunhos", { caminho })
}

export function conferirIso(pacoteId: string, iso: string): Promise<Conferencia> {
  return invoke("conferir_iso", { pacoteId, iso })
}

export function traduzir(pacoteId: string, iso: string, saida: string, sobrescrever: boolean): Promise<Resultado> {
  return invoke("traduzir", { pacoteId, iso, saida, sobrescrever })
}

export function cancelarTraducao(): Promise<void> {
  return invoke("cancelar_traducao")
}

export function arquivoExiste(caminho: string): Promise<boolean> {
  return invoke("arquivo_existe", { caminho })
}

export function abrirPasta(caminho: string): Promise<void> {
  return invoke("abrir_pasta", { caminho })
}

/** Procura uma versão nova do Tradumix no GitHub. Sem versão nova, sem internet ou no npm run dev: null. */
export function procurarVersaoNova(): Promise<VersaoNova | null> {
  return invoke<VersaoNova | null>("procurar_versao_nova").catch(() => null)
}

/** Baixa e instala a versão encontrada (o app reinicia); na portátil, abre a página de download. */
export function instalarVersaoNova(): Promise<void> {
  return invoke("instalar_versao_nova")
}

// A janela não tem a barra do Windows: o topo do app tem os próprios botões.
export function minimizarJanela(): Promise<void> {
  return getCurrentWindow().minimize()
}

export function alternarMaximizarJanela(): Promise<void> {
  return getCurrentWindow().toggleMaximize()
}

export function fecharJanela(): Promise<void> {
  return getCurrentWindow().close()
}

/** Chama `cb` a cada aviso de progresso; devolve a função que para de ouvir. */
export function aoProgresso(cb: (p: Progresso) => void): () => void {
  let parar: (() => void) | null = null
  let cancelado = false
  void listen<Progresso>("progresso", (e) => cb(e.payload)).then((fn) => {
    if (cancelado) fn()
    else parar = fn
  })
  return () => {
    cancelado = true
    parar?.()
  }
}

export async function escolherArquivoIso(): Promise<string | null> {
  const escolha = await open({ multiple: false, directory: false, filters: [{ name: "ISO de PS2", extensions: ["iso"] }] })
  return typeof escolha === "string" ? escolha : null
}

export async function escolherPasta(): Promise<string | null> {
  const escolha = await open({ multiple: false, directory: true })
  return typeof escolha === "string" ? escolha : null
}
