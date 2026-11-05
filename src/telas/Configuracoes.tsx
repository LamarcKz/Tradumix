interface Props {
  mostrarTrailers: boolean
  aoAlterarTrailers: (mostrar: boolean) => void
  aoAbrirSobre: () => void
  aoVoltar: () => void
}

export function Configuracoes({ mostrarTrailers, aoAlterarTrailers, aoAbrirSobre, aoVoltar }: Props) {
  return (
    <section className="max-w-2xl px-10 pb-16 pt-6">
      <button onClick={aoVoltar} className="text-sm text-suave hover:text-texto">
        ← Traduções
      </button>
      <h1 className="titulo mt-8 text-5xl">Configurações</h1>

      <div className="mt-10 border-t border-borda pt-6">
        <div className="flex items-start gap-3">
          <input
            id="opcao-trailer"
            type="checkbox"
            checked={mostrarTrailers}
            onChange={(e) => aoAlterarTrailers(e.target.checked)}
            aria-describedby="opcao-trailer-ajuda"
            className="mt-1 h-4 w-4 accent-[#f2f2f2]"
          />
          <div>
            <label htmlFor="opcao-trailer" className="font-semibold">
              Mostrar o trailer atrás do título
            </label>
            <p id="opcao-trailer-ajuda" className="mt-1 text-sm text-suave">
              Na página do jogo, o trailer do YouTube toca sem som. Desligado, fica só a capa ao fundo.
            </p>
          </div>
        </div>
      </div>

      <div className="mt-10 border-t border-borda pt-6">
        <button onClick={aoAbrirSobre} className="rounded-md border border-borda px-4 py-2 hover:border-destaque">
          Sobre o Tradumix
        </button>
        <p className="mt-2 text-sm text-suave">Versão, licenças e créditos.</p>
      </div>
    </section>
  )
}
