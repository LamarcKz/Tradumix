import type { ListaJogos } from "../tipos"
import { Caixa } from "./Caixa"

interface Props {
  lista: ListaJogos
  aoAbrirJogo: (id: string) => void
  aoEscolherPasta: () => void
}

export function Inicio({ lista, aoAbrirJogo, aoEscolherPasta }: Props) {
  const comAtualizacao = new Set(lista.atualizacoes.map((a) => a.pacote_id))
  const n = lista.jogos.length

  if (!lista.pasta_rascunhos) {
    return (
      <section className="mx-auto flex max-w-xl flex-col items-start gap-5 px-10 pt-24">
        <h1 className="titulo text-5xl">Nenhuma tradução ainda</h1>
        <p className="text-suave">
          Nesta versão, o Tradumix lê as traduções de uma pasta do seu computador. Escolha a pasta com os pacotes de
          rascunho para testar.
        </p>
        <button
          onClick={aoEscolherPasta}
          className="rounded-md bg-destaque px-5 py-3 font-semibold text-fundo hover:bg-[#d5d5d5]"
        >
          Escolher pasta de rascunhos (modo de teste)
        </button>
      </section>
    )
  }

  return (
    <section className="px-10 pb-16 pt-10">
      <header className="mb-8 flex items-end justify-between gap-6">
        <div>
          <h1 className="titulo text-5xl">Traduções</h1>
          <p className="mt-1 text-suave">{n === 1 ? "1 jogo" : `${n} jogos`} para jogar em português</p>
        </div>
      </header>

      {lista.invalidos.length > 0 && (
        <details className="mb-8 rounded-md border border-erro/40 bg-erro/5 px-4 py-3 text-sm">
          <summary className="cursor-pointer text-erro">
            {lista.invalidos.length === 1 ? "1 pacote com problema" : `${lista.invalidos.length} pacotes com problema`}
          </summary>
          <ul className="mt-2 space-y-1 text-suave">
            {lista.invalidos.map((i) => (
              <li key={i.pasta}>
                <span className="text-texto">{i.pasta}</span>: {i.motivo}
              </li>
            ))}
          </ul>
        </details>
      )}

      {n === 0 ? (
        <div className="flex flex-col items-start gap-4 pt-8">
          <p className="text-suave">Nenhuma tradução nesta pasta.</p>
          <button onClick={aoEscolherPasta} className="rounded-md border border-borda px-4 py-2 hover:border-destaque">
            Escolher outra pasta
          </button>
        </div>
      ) : (
        <ul className="grid grid-cols-[repeat(auto-fill,minmax(176px,1fr))] gap-x-7 gap-y-10">
          {lista.jogos.map((jogo) => {
            const atualizar = jogo.traducoes.some((t) => comAtualizacao.has(t.id))
            const beta = jogo.traducoes.every((t) => t.estado !== "publicada")
            const idiomas = [...new Set(jogo.traducoes.map((t) => t.idioma.toUpperCase()))]
            return (
              <li key={jogo.id}>
                <button
                  aria-label={`Abrir ${jogo.nome}`}
                  onClick={() => aoAbrirJogo(jogo.id)}
                  className="group block w-full text-left"
                >
                  <div className="transition-transform duration-200 group-hover:-translate-y-1">
                    <Caixa jogo={jogo} />
                  </div>
                  <p className="mt-3 font-semibold leading-tight">{jogo.nome}</p>
                  <p className="mt-1 flex flex-wrap gap-1.5 text-xs">
                    {idiomas.map((i) => (
                      <span key={i} className="rounded bg-destaque px-1.5 py-0.5 font-semibold text-fundo">
                        {i}
                      </span>
                    ))}
                    {beta && <span className="rounded border border-borda px-1.5 py-0.5 text-suave">Em teste</span>}
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
