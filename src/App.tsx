import { useCallback, useEffect, useState } from "react"

import * as api from "./api"
import { Configuracoes } from "./telas/Configuracoes"
import { Inicio } from "./telas/Inicio"
import { JogoTela, type PedidoTraducao } from "./telas/Jogo"
import { ProgressoTela } from "./telas/Progresso"
import { Pronto } from "./telas/Pronto"
import type { ListaJogos, Progresso, Resultado } from "./tipos"

type Tela =
  | { nome: "inicio" }
  | { nome: "config" }
  | { nome: "jogo"; jogoId: string; aviso?: string; pedido?: PedidoTraducao }
  | { nome: "progresso"; jogoId: string }
  | { nome: "pronto"; jogoId: string; pacoteId: string; resultado: Resultado }

const VAZIA: ListaJogos = { jogos: [], invalidos: [], atualizacoes: [], pasta_rascunhos: null }

export default function App() {
  const [lista, setLista] = useState<ListaJogos | null>(null)
  const [erro, setErro] = useState<string | null>(null)
  const [tela, setTela] = useState<Tela>({ nome: "inicio" })
  const [progresso, setProgresso] = useState<Progresso>({ etapa: "conferindo", fracao: 0 })

  const recarregar = useCallback(async () => {
    try {
      setLista(await api.listarJogos())
    } catch (e) {
      setErro(String(e))
      setLista(VAZIA)
    }
  }, [])

  useEffect(() => {
    void recarregar()
  }, [recarregar])

  async function trocarPasta() {
    const pasta = await api.escolherPasta()
    if (!pasta) return
    try {
      setLista(await api.escolherPastaRascunhos(pasta))
      setErro(null)
    } catch (e) {
      setErro(String(e))
    }
  }

  async function traduzir(jogoId: string, pedido: PedidoTraducao) {
    setProgresso({ etapa: "conferindo", fracao: 0 })
    setTela({ nome: "progresso", jogoId })
    const parar = api.aoProgresso(setProgresso)
    try {
      const resultado = await api.traduzir(pedido.pacoteId, pedido.iso, pedido.saida, pedido.sobrescrever)
      setTela({ nome: "pronto", jogoId, pacoteId: pedido.pacoteId, resultado })
      void recarregar()
    } catch (e) {
      setTela({ nome: "jogo", jogoId, aviso: String(e), pedido })
    } finally {
      parar()
    }
  }

  if (!lista) return <div className="p-10 text-suave">Carregando…</div>

  const jogo = "jogoId" in tela ? lista.jogos.find((j) => j.id === tela.jogoId) : undefined
  const ocupado = tela.nome === "progresso"

  return (
    <div className="flex h-full flex-col">
      <header className="flex h-14 shrink-0 items-center justify-between border-b border-borda/60 px-10">
        <button onClick={() => !ocupado && setTela({ nome: "inicio" })} className="titulo text-xl" disabled={ocupado}>
          Tradumix
        </button>
        {!ocupado && tela.nome !== "config" && (
          <button onClick={() => setTela({ nome: "config" })} className="text-sm text-suave hover:text-texto">
            Configurações
          </button>
        )}
      </header>

      {erro && (
        <p role="alert" className="mx-10 mt-4 rounded-md bg-erro/10 p-3 text-sm text-erro">
          {erro}
        </p>
      )}

      <main className="min-h-0 flex-1 overflow-y-auto">
        {tela.nome === "inicio" && (
          <Inicio lista={lista} aoAbrirJogo={(id) => setTela({ nome: "jogo", jogoId: id })} aoEscolherPasta={trocarPasta} />
        )}
        {tela.nome === "config" && (
          <Configuracoes pasta={lista.pasta_rascunhos} aoTrocarPasta={trocarPasta} aoVoltar={() => setTela({ nome: "inicio" })} />
        )}
        {tela.nome === "jogo" && jogo && (
          <JogoTela
            key={jogo.id}
            jogo={jogo}
            aviso={tela.aviso}
            pedidoAnterior={tela.pedido}
            aoVoltar={() => setTela({ nome: "inicio" })}
            aoTraduzir={(pedido) => void traduzir(jogo.id, pedido)}
          />
        )}
        {tela.nome === "progresso" && jogo && <ProgressoTela nomeJogo={jogo.nome} progresso={progresso} />}
        {tela.nome === "pronto" && jogo && (
          <Pronto
            jogo={jogo}
            traducao={jogo.traducoes.find((t) => t.id === tela.pacoteId) ?? jogo.traducoes[0]}
            resultado={tela.resultado}
            aoVoltar={() => setTela({ nome: "inicio" })}
          />
        )}
      </main>
    </div>
  )
}
