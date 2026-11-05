/** O T do Tradumix: barra cortada por uma fenda diagonal e haste solta, como um patch encaixado. */
export function Marca({ tamanho = 28 }: { tamanho?: number }) {
  return (
    <svg viewBox="226 238 572 548" width={tamanho} height={tamanho} aria-hidden fill="currentColor">
      <path d="M226 238 H586 L500 352 H226 Z" />
      <path d="M646 238 H798 V352 H560 Z" />
      <path d="M456 398 H568 V786 H456 Z" />
    </svg>
  )
}

export function Logo() {
  return (
    <span className="flex items-center gap-3">
      <Marca tamanho={24} />
      <span className="titulo text-lg">Tradumix</span>
    </span>
  )
}
