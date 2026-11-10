import { useState } from "react"

import type { InfoPlataforma, Jogo, ListaJogos } from "../tipos"
import { Caixa } from "./Caixa"

interface Props {
  lista: ListaJogos
  /** O app está baixando o catálogo agora. */
  sincronizando?: boolean
  /** `plataformaId` vem quando o card foi aberto com um filtro escolhido. */
  aoAbrirJogo: (id: string, plataformaId?: string) => void
  aoTentarDeNovo: () => void
}

/** As plataformas que aparecem nos jogos, na ordem em que aparecem. */
function plataformasDaLista(jogos: Jogo[]): InfoPlataforma[] {
  const vistas = new Map<string, InfoPlataforma>()
  for (const j of jogos) for (const p of j.plataformas) if (!vistas.has(p.info.id)) vistas.set(p.info.id, p.info)
  return [...vistas.values()]
}

export function Inicio({ lista, sincronizando = false, aoAbrirJogo, aoTentarDeNovo }: Props) {
  const [filtro, setFiltro] = useState<string | null>(null)
  const comAtualizacao = new Set(lista.atualizacoes.map((a) => a.pacote_id))
  const plataformas = plataformasDaLista(lista.jogos)
  // O filtro some se a plataforma escolhida saiu do catálogo.
  const ativo = filtro && plataformas.some((p) => p.id === filtro) ? filtro : null
  const jogos = ativo ? lista.jogos.filter((j) => j.plataformas.some((p) => p.info.id === ativo)) : lista.jogos
  const n = lista.jogos.length

  const botaoFiltro = (selecionado: boolean) =>
    `rounded-full px-3 py-1 text-sm ${selecionado ? "bg-destaque font-semibold text-fundo" : "border border-borda text-suave hover:text-texto"}`

  return (
    <section className="px-10 pb-16 pt-10">
      <header className="mb-8 flex items-end justify-between gap-6">
        <div>
          <h1 className="titulo text-5xl">Traduções</h1>
          {n > 0 && <p className="mt-1 text-suave">{n === 1 ? "1 jogo" : `${n} jogos`} para jogar em português</p>}
        </div>
        {sincronizando && <p className="text-sm text-sutil">Atualizando o catálogo…</p>}
      </header>

      {lista.catalogo.aviso && (
        <p role="status" className="mb-8 rounded-md border border-borda bg-superficie px-4 py-3 text-sm text-suave">
          {lista.catalogo.aviso}
        </p>
      )}

      {plataformas.length > 1 && (
        <div role="group" aria-label="Filtrar por plataforma" className="mb-8 flex flex-wrap gap-2">
          <button aria-pressed={ativo === null} onClick={() => setFiltro(null)} className={botaoFiltro(ativo === null)}>
            Todas
          </button>
          {plataformas.map((p) => (
            <button key={p.id} aria-pressed={ativo === p.id} onClick={() => setFiltro(p.id)} className={botaoFiltro(ativo === p.id)}>
              {p.sigla}
            </button>
          ))}
        </div>
      )}

      {n === 0 ? (
        !sincronizando && (
          <div className="flex flex-col items-start gap-4 pt-16">
            <p className="titulo text-3xl">Nenhuma tradução disponível ainda.</p>
            <p className="max-w-md text-suave">
              As traduções aparecem aqui assim que o catálogo é baixado. Confira a conexão e tente de novo.
            </p>
            <button onClick={aoTentarDeNovo} className="rounded-md bg-destaque px-5 py-3 font-semibold text-fundo hover:bg-[#d5d5d5]">
              Tentar de novo
            </button>
          </div>
        )
      ) : (
        <ul className="grid grid-cols-[repeat(auto-fill,minmax(176px,1fr))] gap-x-7 gap-y-10">
          {jogos.map((jogo) => {
            const plataforma = jogo.plataformas.find((p) => p.info.id === ativo) ?? jogo.plataformas[0]
            const traducoes = jogo.plataformas.flatMap((p) => p.traducoes)
            const atualizar = traducoes.some((t) => comAtualizacao.has(t.id))
            const novo = traducoes.some((t) => t.novo)
            const idiomas = [...new Set(traducoes.map((t) => t.idioma.toUpperCase()))]
            const abrir = () => (ativo ? aoAbrirJogo(jogo.id, ativo) : aoAbrirJogo(jogo.id))
            return (
              <li key={jogo.id}>
                <button aria-label={`Abrir ${jogo.nome}`} onClick={abrir} className="group block w-full text-left">
                  <div className="transition-transform duration-200 group-hover:-translate-y-1">
                    <Caixa nome={jogo.nome} capa={plataforma.capa} plataforma={plataforma.info} />
                  </div>
                  <p className="mt-3 font-semibold leading-tight">{jogo.nome}</p>
                  <p className="mt-1 text-xs text-suave">{jogo.plataformas.map((p) => p.info.sigla).join(" · ")}</p>
                  <p className="mt-1 flex flex-wrap gap-1.5 text-xs">
                    {idiomas.map((i) => (
                      <span key={i} className="rounded bg-destaque px-1.5 py-0.5 font-semibold text-fundo">
                        {i}
                      </span>
                    ))}
                    {novo && <span className="rounded bg-verde/15 px-1.5 py-0.5 font-semibold text-verde">Novo</span>}
                    {atualizar && <span className="rounded bg-verde/15 px-1.5 py-0.5 text-verde">Atualização disponível</span>}
                  </p>
                </button>
              </li>
            )
          })}
        </ul>
      )}
    </section>
  )
}
