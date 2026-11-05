// A única ponte entre as telas e o app em Rust. Os testes trocam este módulo por um falso.
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { open } from "@tauri-apps/plugin-dialog"

import type { Conferencia, ListaJogos, Progresso, Resultado } from "./tipos"

export function listarJogos(): Promise<ListaJogos> {
  return invoke("listar_jogos")
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

export function abrirPasta(caminho: string): Promise<void> {
  return invoke("abrir_pasta", { caminho })
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
