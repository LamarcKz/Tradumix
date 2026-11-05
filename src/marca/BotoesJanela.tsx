import * as api from "../api"

/** Minimizar, maximizar e fechar, no topo do app (a janela não usa a barra do Windows). */
export function BotoesJanela() {
  const botao = "flex h-full w-12 items-center justify-center text-suave transition-colors hover:bg-white/[0.08] hover:text-texto"
  return (
    <div className="flex h-full self-stretch">
      <button aria-label="Minimizar" onClick={() => void api.minimizarJanela()} className={botao}>
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
          <path d="M0 5h10" stroke="currentColor" strokeWidth="1" />
        </svg>
      </button>
      <button aria-label="Maximizar" onClick={() => void api.alternarMaximizarJanela()} className={botao}>
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
          <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" strokeWidth="1" />
        </svg>
      </button>
      <button
        aria-label="Fechar"
        onClick={() => void api.fecharJanela()}
        className="flex h-full w-12 items-center justify-center text-suave transition-colors hover:bg-[#c42b1c] hover:text-white"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
          <path d="M0 0l10 10M10 0L0 10" stroke="currentColor" strokeWidth="1" />
        </svg>
      </button>
    </div>
  )
}
