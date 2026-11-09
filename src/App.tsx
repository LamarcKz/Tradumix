import { useCallback, useEffect, useState } from "react"

import * as api from "./api"
import { BotoesJanela } from "./marca/BotoesJanela"
import { Logo } from "./marca/Logo"
import { MarcaDev } from "./marca/MarcaDev"
import { MarcaPrevia } from "./marca/MarcaPrevia"
import { AvisoVersao } from "./telas/AvisoVersao"
import { Configuracoes } from "./telas/Configuracoes"
import { Sobre } from "./telas/Sobre"
import { Inicio } from "./telas/Inicio"
import { JogoTela, type PedidoTraducao } from "./telas/Jogo"
import { ProgressoTela } from "./telas/Progresso"
import { Pronto } from "./telas/Pronto"
import type { ListaJogos, Progresso, Resultado } from "./tipos"

type Tela =
  | { nome: "inicio" }
  | { nome: "config" }
  | { nome: "sobre" }
  | { nome: "jogo"; jogoId: string; plataformaId?: string; aviso?: string; pedido?: PedidoTraducao }
  | { nome: "progresso"; jogoId: string; plataformaId: string; comDownload: boolean }
  | { nome: "pronto"; jogoId: string; plataformaId: string; pacoteId: string; resultado: Resultado }

const VAZIA: ListaJogos = {
  jogos: [],
  atualizacoes: [],
  catalogo: { offline: false, atualizado_em: null, aviso: null },
  mostrar_trailers: true,
}

export default function App() {
  const [lista, setLista] = useState<ListaJogos | null>(null)
  const [erro, setErro] = useState<string | null>(null)
  const [tela, setTela] = useState<Tela>({ nome: "inicio" })
  const [progresso, setProgresso] = useState<Progresso>({ etapa: "conferindo", fracao: 0 })
  const [sincronizando, setSincronizando] = useState(false)

  const recarregar = useCallback(async () => {
    try {
      setLista(await api.listarJogos())
    } catch (e) {
      setErro(String(e))
      setLista(VAZIA)
    }
  }, [])

  // Baixa o catálogo do GitHub; sem internet, o app fica com o que já estava no cache.
  const sincronizar = useCallback(async () => {
    setSincronizando(true)
    try {
      setLista(await api.sincronizarCatalogo())
    } catch (e) {
      setErro(String(e))
    } finally {
      setSincronizando(false)
    }
  }, [])

  // Mostra o cache na hora e atualiza em seguida.
  useEffect(() => {
    void recarregar().then(sincronizar)
  }, [recarregar, sincronizar])

  async function alterarTrailers(mostrar: boolean) {
    try {
      setLista(await api.definirMostrarTrailers(mostrar))
    } catch (e) {
      setErro(String(e))
    }
  }

  async function traduzir(jogoId: string, pedido: PedidoTraducao) {
    setProgresso({ etapa: "conferindo", fracao: 0 })
    const plataforma = lista?.jogos.find((j) => j.id === jogoId)?.plataformas.find((p) => p.info.id === pedido.plataformaId)
    const traducao = plataforma?.traducoes.find((t) => t.id === pedido.pacoteId)
    const { plataformaId } = pedido
    setTela({ nome: "progresso", jogoId, plataformaId, comDownload: traducao ? !traducao.patch_disponivel : false })
    const parar = api.aoProgresso(setProgresso)
    try {
      const resultado = await api.traduzir(pedido.pacoteId, pedido.iso, pedido.saida, pedido.sobrescrever)
      setTela({ nome: "pronto", jogoId, plataformaId, pacoteId: pedido.pacoteId, resultado })
      void recarregar()
    } catch (e) {
      setTela({ nome: "jogo", jogoId, aviso: String(e), pedido })
    } finally {
      parar()
    }
  }

  if (!lista) return <div className="p-10 text-suave">Carregando…</div>

  const jogo = "jogoId" in tela ? lista.jogos.find((j) => j.id === tela.jogoId) : undefined
  // O jogo aberto saiu do catálogo na última atualização: volta para o início em vez de ficar em branco.
  if ("jogoId" in tela && !jogo && tela.nome !== "progresso") {
    setTela({ nome: "inicio" })
    return null
  }
  const ocupado = tela.nome === "progresso"
  // A plataforma da tela de progresso ou de pronto (a primeira do jogo, se ela tiver saído do catálogo).
  const plataforma =
    jogo && "plataformaId" in tela ? (jogo.plataformas.find((p) => p.info.id === tela.plataformaId) ?? jogo.plataformas[0]) : undefined

  return (
    <div className="flex h-full flex-col">
      {/* O topo é a barra da janela: arrasta a janela (data-tauri-drag-region) e tem os botões dela. */}
      <header
        data-tauri-drag-region
        className="flex h-14 shrink-0 select-none items-center border-b border-white/[0.05] bg-fundo pl-10"
      >
        <button
          onClick={() => !ocupado && setTela({ nome: "inicio" })}
          aria-label="Tradumix, voltar ao início"
          disabled={ocupado}
          className="relative"
        >
          <Logo />
          <MarcaDev />
        </button>
        <div data-tauri-drag-region className="h-full flex-1" />
        {!ocupado && tela.nome !== "config" && (
          <button onClick={() => setTela({ nome: "config" })} className="mr-6 text-sm text-suave hover:text-texto">
            Configurações
          </button>
        )}
        <BotoesJanela />
      </header>

      <MarcaPrevia />
      <AvisoVersao />

      {erro && (
        <p role="alert" className="mx-10 mt-4 rounded-md bg-erro/10 p-3 text-sm text-erro">
          {erro}
        </p>
      )}

      <main className="min-h-0 flex-1 overflow-y-auto">
        {tela.nome === "inicio" && (
          <Inicio
            lista={lista}
            sincronizando={sincronizando}
            aoAbrirJogo={(id, plataformaId) => setTela({ nome: "jogo", jogoId: id, plataformaId })}
            aoTentarDeNovo={() => void sincronizar()}
          />
        )}
        {tela.nome === "config" && (
          <Configuracoes
            mostrarTrailers={lista.mostrar_trailers}
            aoAlterarTrailers={(m) => void alterarTrailers(m)}
            aoAbrirSobre={() => setTela({ nome: "sobre" })}
            aoVoltar={() => setTela({ nome: "inicio" })}
          />
        )}
        {tela.nome === "sobre" && <Sobre aoVoltar={() => setTela({ nome: "config" })} />}
        {tela.nome === "jogo" && jogo && (
          <JogoTela
            key={jogo.id}
            jogo={jogo}
            plataformaInicial={tela.plataformaId}
            aviso={tela.aviso}
            offline={lista.catalogo.offline}
            mostrarTrailers={lista.mostrar_trailers}
            pedidoAnterior={tela.pedido}
            aoVoltar={() => setTela({ nome: "inicio" })}
            aoTraduzir={(pedido) => void traduzir(jogo.id, pedido)}
          />
        )}
        {tela.nome === "progresso" && jogo && plataforma && (
          <ProgressoTela nomeJogo={jogo.nome} plataforma={plataforma.info} progresso={progresso} comDownload={tela.comDownload} />
        )}
        {tela.nome === "pronto" && jogo && plataforma && (
          <Pronto
            nomeJogo={jogo.nome}
            plataforma={plataforma}
            traducao={plataforma.traducoes.find((t) => t.id === tela.pacoteId) ?? plataforma.traducoes[0]}
            resultado={tela.resultado}
            aoVoltar={() => setTela({ nome: "inicio" })}
          />
        )}
      </main>
    </div>
  )
}
