import * as api from "../api"
import type { Etapa, Progresso } from "../tipos"

const ETAPAS: { etapa: Etapa; nome: string }[] = [
  { etapa: "conferindo", nome: "Conferindo a ISO" },
  { etapa: "aplicando", nome: "Aplicando a tradução" },
  { etapa: "verificando", nome: "Verificando o hash do RetroAchievements" },
]

export function ProgressoTela({ nomeJogo, progresso }: { nomeJogo: string; progresso: Progresso }) {
  const atual = ETAPAS.findIndex((e) => e.etapa === progresso.etapa)
  const pct = Math.round(Math.min(Math.max(progresso.fracao, 0), 1) * 100)

  return (
    <section className="mx-auto flex max-w-2xl flex-col gap-10 px-10 pt-24">
      <div>
        <p className="text-suave">Traduzindo</p>
        <h1 className="titulo text-6xl">{nomeJogo}</h1>
      </div>

      <ol className="space-y-4">
        {ETAPAS.map((e, i) => {
          const feita = i < atual
          const agora = i === atual
          return (
            <li key={e.etapa} aria-current={agora ? "step" : undefined} className="flex items-center gap-4">
              <span
                className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-full border text-sm font-semibold ${
                  feita ? "border-verde bg-verde text-fundo" : agora ? "border-destaque text-destaque" : "border-borda text-suave"
                }`}
              >
                {feita ? "✓" : i + 1}
              </span>
              <span className={agora ? "text-lg font-semibold" : feita ? "text-texto" : "text-suave"}>{e.nome}</span>
            </li>
          )
        })}
      </ol>

      <div>
        <div className="flex items-baseline justify-between">
          <span className="text-suave">
            Etapa {atual + 1} de {ETAPAS.length}
          </span>
          <span className="titulo text-4xl tabular-nums">{pct}%</span>
        </div>
        <div
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={pct}
          aria-label={ETAPAS[atual]?.nome}
          className="mt-3 h-3 overflow-hidden rounded-full bg-superficie"
        >
          <div className="h-full rounded-full bg-destaque transition-[width] duration-200" style={{ width: `${pct}%` }} />
        </div>
      </div>

      <button
        onClick={() => void api.cancelarTraducao()}
        className="self-start rounded-md border border-borda px-4 py-2 text-suave hover:border-erro hover:text-erro"
      >
        Cancelar
      </button>
    </section>
  )
}
