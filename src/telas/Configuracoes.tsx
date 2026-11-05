interface Props {
  pasta: string | null
  aoTrocarPasta: () => void
  aoVoltar: () => void
}

export function Configuracoes({ pasta, aoTrocarPasta, aoVoltar }: Props) {
  return (
    <section className="max-w-2xl px-10 pb-16 pt-6">
      <button onClick={aoVoltar} className="text-sm text-suave hover:text-texto">
        ← Traduções
      </button>
      <h1 className="titulo mt-8 text-5xl">Configurações</h1>

      <div className="mt-10 border-t border-borda pt-6">
        <h2 className="font-semibold">Pasta de rascunhos (modo de teste)</h2>
        <p className="mt-1 text-sm text-suave">
          O Tradumix mostra as traduções que estão nesta pasta. Cada tradução fica numa subpasta com o ficha.json e o
          patch.
        </p>
        <p className="mt-4 break-all rounded-md bg-superficie px-4 py-3 text-sm">{pasta ?? "Nenhuma pasta escolhida"}</p>
        <button onClick={aoTrocarPasta} className="mt-3 rounded-md border border-borda px-4 py-2 hover:border-amarelo">
          {pasta ? "Trocar pasta" : "Escolher pasta"}
        </button>
      </div>
    </section>
  )
}
