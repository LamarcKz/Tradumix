/** Faixa no topo da prévia (build `npm run build:previa`), para ninguém confundir com o app. */
export function MarcaPrevia({ ativa = import.meta.env.MODE === "previa" }: { ativa?: boolean }) {
  if (!ativa) return null
  return (
    <p role="note" className="shrink-0 bg-dev px-10 py-1.5 text-center text-sm font-semibold text-black">
      Prévia: é assim que a tradução vai aparecer no Tradumix
    </p>
  )
}
