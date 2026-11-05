import * as api from "../api"
import type { Jogo, Resultado, Traducao } from "../tipos"
import { Caixa } from "./Caixa"

interface Props {
  jogo: Jogo
  traducao: Traducao
  resultado: Resultado
  aoVoltar: () => void
}

export function Pronto({ jogo, traducao, resultado, aoVoltar }: Props) {
  const segundos = Math.max(1, Math.round(resultado.segundos))
  return (
    <section className="mx-auto grid max-w-4xl grid-cols-[auto_1fr] items-start gap-12 px-10 pt-20">
      <Caixa jogo={jogo} tamanho="grande" />
      <div className="min-w-0">
        <h1 className="titulo text-6xl">ISO traduzida pronta</h1>
        <p className="mt-3 text-suave">
          {jogo.nome} em {traducao.idioma.toUpperCase()}, versão {traducao.versao}, gerada em {segundos} s.
        </p>

        <p className="mt-6 break-all rounded-md bg-superficie px-4 py-3 text-sm">{resultado.saida}</p>

        {resultado.compativel_ra ? (
          <p className="mt-4 flex items-center gap-2 font-semibold text-verde">
            <span aria-hidden className="h-2.5 w-2.5 rounded-full bg-verde" />
            Compatível com RetroAchievements
          </p>
        ) : (
          <p className="mt-4 text-amarelo">O hash do RetroAchievements mudou: as conquistas podem não funcionar.</p>
        )}

        <div className="mt-8">
          <h2 className="font-semibold">Tradução feita por</h2>
          <ul className="mt-1 text-suave">
            {traducao.tradutores.map((t) => (
              <li key={t.nome}>{t.nome}</li>
            ))}
          </ul>
        </div>

        <div className="mt-10 flex gap-3">
          <button
            onClick={() => void api.abrirPasta(resultado.saida)}
            className="rounded-md bg-amarelo px-5 py-3 font-semibold text-fundo hover:brightness-110"
          >
            Abrir pasta
          </button>
          <button onClick={aoVoltar} className="rounded-md border border-borda px-5 py-3 hover:border-amarelo">
            Voltar ao início
          </button>
        </div>
      </div>
    </section>
  )
}
