import { useEffect, useState } from "react"

import * as api from "../api"
import type { Conferencia, Jogo } from "../tipos"
import { Caixa } from "./Caixa"
import { FundoJogo } from "./FundoJogo"

export interface PedidoTraducao {
  pacoteId: string
  plataformaId: string
  iso: string
  saida: string
  sobrescrever: boolean
}

interface Props {
  jogo: Jogo
  /** Plataforma que já vem escolhida (a do filtro da tela inicial). */
  plataformaInicial?: string
  aviso?: string | null
  /** Sem internet: traduções que ainda não têm o patch no PC não podem ser geradas. */
  offline?: boolean
  /** Toca o trailer do YouTube atrás do título (opção em Configurações). */
  mostrarTrailers?: boolean
  /** O pedido que voltou com erro: a tela reabre com a mesma ISO e a mesma saída. */
  pedidoAnterior?: PedidoTraducao
  aoVoltar: () => void
  aoTraduzir: (pedido: PedidoTraducao) => void
}

/** O texto do link nos créditos: o endereço sem "https://" e sem "www." (ex.: github.com/LamarcKz). */
function textoDoLink(link: string) {
  return link.replace(/^https?:\/\/(www\.)?/, "").replace(/\/$/, "")
}

function nomeDoArquivo(caminho: string) {
  return caminho.split(/[\\/]/).pop() ?? caminho
}

/** 238940 → "233 KB"; 5242880 → "5,0 MB". */
export function tamanhoLegivel(bytes: number) {
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1).replace(".", ",")} MB`
}

export function JogoTela({ jogo, plataformaInicial, aviso, offline = false, mostrarTrailers = true, pedidoAnterior, aoVoltar, aoTraduzir }: Props) {
  const [plataformaId, setPlataformaId] = useState(pedidoAnterior?.plataformaId ?? plataformaInicial ?? jogo.plataformas[0]?.info.id ?? "")
  const plataforma = jogo.plataformas.find((p) => p.info.id === plataformaId) ?? jogo.plataformas[0]
  const arquivo = plataforma.info.arquivo
  const [traducaoId, setTraducaoId] = useState(pedidoAnterior?.pacoteId ?? plataforma.traducoes[0]?.id ?? "")
  const [iso, setIso] = useState<string | null>(null)
  const [conferencia, setConferencia] = useState<Conferencia | null>(null)
  const [saida, setSaida] = useState<string | null>(null)
  const [erro, setErro] = useState<string | null>(null)
  const [conferindo, setConferindo] = useState(false)
  const traducao = plataforma.traducoes.find((t) => t.id === traducaoId) ?? plataforma.traducoes[0]
  const semInternet = offline && !traducao.patch_disponivel
  const temTrailer = mostrarTrailers && !offline && !!traducao.trailer_youtube

  // Voltando de um erro: confere de novo a mesma ISO e mantém a saída escolhida.
  useEffect(() => {
    if (!pedidoAnterior) return
    let ativo = true
    void (async () => {
      try {
        const c = await api.conferirIso(pedidoAnterior.pacoteId, pedidoAnterior.iso)
        const existe = c.serve ? await api.arquivoExiste(pedidoAnterior.saida) : false
        if (!ativo) return
        setIso(pedidoAnterior.iso)
        setConferencia({ ...c, ja_existe: existe })
        setSaida(pedidoAnterior.saida)
      } catch (e) {
        if (ativo) setErro(String(e))
      }
    })()
    return () => {
      ativo = false
    }
  }, [pedidoAnterior])

  /** Outra plataforma: outra lista de traduções e outro arquivo; o que foi escolhido antes não serve. */
  function trocarPlataforma(id: string) {
    const nova = jogo.plataformas.find((p) => p.info.id === id)
    if (!nova) return
    setPlataformaId(id)
    setTraducaoId(nova.traducoes[0]?.id ?? "")
    setIso(null)
    setConferencia(null)
    setSaida(null)
    setErro(null)
  }

  async function selecionarIso() {
    const escolhida = await api.escolherArquivoDoJogo(plataforma.info)
    if (!escolhida) return
    setErro(null)
    setConferindo(true)
    try {
      const c = await api.conferirIso(traducao.id, escolhida)
      setIso(escolhida)
      setConferencia(c)
      setSaida(c.saida_sugerida)
    } catch (e) {
      setErro(String(e))
      setConferencia(null)
    } finally {
      setConferindo(false)
    }
  }

  async function escolherOutraPasta() {
    const pasta = await api.escolherPasta()
    if (!pasta || !conferencia) return
    const nova = `${pasta.replace(/[\\/]$/, "")}\\${nomeDoArquivo(conferencia.saida_sugerida)}`
    const existe = await api.arquivoExiste(nova)
    setSaida(nova)
    setConferencia({ ...conferencia, ja_existe: existe })
  }

  function traduzir(sobrescrever: boolean) {
    if (!iso || !saida) return
    aoTraduzir({ pacoteId: traducao.id, plataformaId: plataforma.info.id, iso, saida, sobrescrever })
  }

  return (
    <div className="relative min-h-full">
      <FundoJogo nome={jogo.nome} capa={plataforma.capa} traducao={traducao} mostrarTrailers={mostrarTrailers} offline={offline} />

      <div className="relative px-10 pb-16 pt-6">
        <button
          onClick={aoVoltar}
          className="rounded-md bg-fundo/60 px-3 py-1.5 text-sm text-suave backdrop-blur hover:text-texto"
        >
          ← Traduções
        </button>

        {/* Com trailer, o vídeo ocupa o topo e as informações começam na base dele. */}
        <div className={`grid grid-cols-[auto_1fr_340px] items-start gap-10 ${temTrailer ? "mt-[38vh]" : "mt-8"}`}>
          <Caixa nome={jogo.nome} capa={plataforma.capa} plataforma={plataforma.info} tamanho="grande" />

          <article className="min-w-0 max-w-[64ch]">
            {jogo.plataformas.length > 1 && (
              <div role="tablist" aria-label="Plataforma" className="mb-5 flex flex-wrap gap-2">
                {jogo.plataformas.map((p) => {
                  const escolhida = p.info.id === plataforma.info.id
                  return (
                    <button
                      key={p.info.id}
                      role="tab"
                      aria-selected={escolhida}
                      onClick={() => trocarPlataforma(p.info.id)}
                      className={`rounded-md px-3 py-1.5 text-sm backdrop-blur ${
                        escolhida ? "bg-destaque font-semibold text-fundo" : "bg-fundo/60 text-suave hover:text-texto"
                      }`}
                    >
                      {p.info.nome}
                    </button>
                  )
                })}
              </div>
            )}
            <h1 className="titulo text-6xl">{jogo.nome}</h1>
            <p className="mt-3 text-suave">
              Tradução {traducao.idioma.toUpperCase()} versão {traducao.versao}, para a {traducao.detalhes}
            </p>

            {plataforma.traducoes.length > 1 && (
              <fieldset className="mt-6">
                <legend className="mb-2 font-semibold">Traduções disponíveis</legend>
                <div className="flex flex-col gap-2">
                  {plataforma.traducoes.map((t) => (
                    <label key={t.id} className="flex items-center gap-2">
                      <input
                        type="radio"
                        name="traducao"
                        checked={t.id === traducao.id}
                        onChange={() => {
                          setTraducaoId(t.id)
                          setConferencia(null)
                          setIso(null)
                        }}
                      />
                      {t.idioma.toUpperCase()} v{t.versao} ({t.detalhes}), por {t.tradutores.map((x) => x.nome).join(", ")}
                    </label>
                  ))}
                </div>
              </fieldset>
            )}

            {traducao.traduzido.length > 0 && (
              <section className="mt-8">
                <h2 className="font-semibold">O que foi traduzido</h2>
                <ul className="mt-2 flex flex-wrap gap-2">
                  {traducao.traduzido.map((item) => (
                    <li key={item} className="rounded-full bg-superficie px-3 py-1 text-sm">
                      {item}
                    </li>
                  ))}
                </ul>
              </section>
            )}

            {traducao.limitacoes.length > 0 && (
              <section className="mt-8">
                <h2 className="font-semibold">O que não foi traduzido</h2>
                <ul className="mt-2 list-disc space-y-1 pl-5 text-suave">
                  {traducao.limitacoes.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              </section>
            )}

            <section className="mt-8">
              <h2 className="font-semibold">Créditos</h2>
              <ul className="mt-2 space-y-1">
                {traducao.tradutores.map((t) => (
                  <li key={t.nome} className="flex flex-wrap items-baseline gap-x-3">
                    <span className="font-semibold">{t.nome}</span>
                    {t.link && (
                      <a href={t.link} target="_blank" rel="noreferrer" className="text-sm text-suave underline decoration-sutil underline-offset-4 hover:text-texto hover:decoration-texto">
                        {textoDoLink(t.link)}
                      </a>
                    )}
                  </li>
                ))}
              </ul>
              {traducao.permissao && <p className="mt-2 text-sm text-suave">Permissão: {traducao.permissao}</p>}
            </section>
          </article>

          <aside className="rounded-lg border border-borda bg-superficie/90 p-6 backdrop-blur">
            <h2 className="titulo text-2xl">Gerar a {arquivo} traduzida</h2>
            <p className="mt-2 text-sm text-suave">{traducao.instrucao}</p>

            {semInternet ? (
              <p role="status" className="mt-4 rounded-md border border-borda p-3 text-sm text-suave">
                Precisa de internet para baixar a tradução.
              </p>
            ) : (
              !traducao.patch_disponivel &&
              traducao.tamanho_download !== null && (
                <p className="mt-3 text-sm text-sutil">Baixa {tamanhoLegivel(traducao.tamanho_download)} da internet na hora de traduzir.</p>
              )
            )}

            {aviso && (
              <p role="status" className="mt-4 rounded-md bg-erro/10 p-3 text-sm text-erro">
                {aviso}
              </p>
            )}

            {conferencia && !conferencia.serve && (
              <p role="alert" className="mt-4 rounded-md bg-erro/10 p-3 text-sm text-erro">
                {conferencia.mensagem}
              </p>
            )}
            {erro && (
              <p role="alert" className="mt-4 rounded-md bg-erro/10 p-3 text-sm text-erro">
                {erro}
              </p>
            )}

            {conferencia?.serve && saida ? (
              <div className="mt-5 space-y-4">
                <div>
                  <p className="text-sm text-suave">{arquivo} original</p>
                  <p className="break-all text-sm">{iso}</p>
                </div>
                <div>
                  <p className="text-sm text-suave">A traduzida será gravada em</p>
                  <p className="break-all text-sm">{saida}</p>
                  <button onClick={escolherOutraPasta} className="mt-1 text-sm text-texto underline decoration-sutil underline-offset-4 hover:decoration-texto">
                    Escolher outra pasta
                  </button>
                </div>
                {conferencia.ja_existe ? (
                  <div className="space-y-2">
                    <p className="text-sm text-erro">Já existe um arquivo com esse nome.</p>
                    <button
                      onClick={() => traduzir(true)}
                      className="w-full rounded-md bg-destaque px-5 py-3 font-semibold text-fundo hover:bg-[#d5d5d5]"
                    >
                      Substituir e traduzir
                    </button>
                  </div>
                ) : (
                  <button
                    onClick={() => traduzir(false)}
                    className="w-full rounded-md bg-destaque px-5 py-3 font-semibold text-fundo hover:bg-[#d5d5d5]"
                  >
                    Traduzir
                  </button>
                )}
                <button onClick={selecionarIso} className="w-full text-sm text-suave hover:text-texto">
                  Usar outra {arquivo}
                </button>
              </div>
            ) : (
              <button
                onClick={selecionarIso}
                disabled={conferindo || semInternet}
                className="mt-5 w-full rounded-md bg-destaque px-5 py-3 font-semibold text-fundo hover:bg-[#d5d5d5] disabled:opacity-60"
              >
                {conferindo ? `Conferindo a ${arquivo}…` : conferencia ? `Selecionar outra ${arquivo}` : `Selecionar ${arquivo}`}
              </button>
            )}
          </aside>
        </div>
      </div>
    </div>
  )
}
