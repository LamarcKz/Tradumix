import type { InfoPlataforma } from "../tipos"

interface Props {
  nome: string
  capa: string | null
  plataforma: InfoPlataforma
  tamanho?: "normal" | "grande"
}

// As classes ficam escritas por inteiro para o Tailwind gerá-las.
const PROPORCOES: Record<string, string> = {
  "7/10": "aspect-[7/10]",
  "1/1": "aspect-square",
  "2/3": "aspect-[2/3]",
}

/**
 * A capa no formato da caixa da plataforma (DVD no PS2, estojo de CD no PS1, capa da Steam no PC),
 * para aparecer inteira. As capas do acervo já trazem a faixa da plataforma; sem capa, o app desenha
 * a faixa com o nome da plataforma e o nome do jogo.
 */
export function Caixa({ nome, capa, plataforma, tamanho = "normal" }: Props) {
  const grande = tamanho === "grande"
  return (
    <div
      className={`relative ${PROPORCOES[plataforma.proporcao_capa] ?? PROPORCOES["7/10"]} overflow-hidden rounded-[6px] bg-caixa shadow-[0_18px_40px_-18px_rgba(0,0,0,0.8)] ring-1 ring-white/10 ${
        grande ? "w-64" : "w-full"
      }`}
    >
      {capa ? (
        <img src={capa} alt="" className="h-full w-full object-cover" />
      ) : (
        <>
          <div className={`flex items-center bg-caixa px-3 ${grande ? "h-9" : "h-7"}`}>
            <span className={`font-semibold tracking-tight text-white/90 ${grande ? "text-sm" : "text-[11px]"}`}>{plataforma.nome}</span>
          </div>
          <div className="flex h-[calc(100%-1.75rem)] items-end bg-gradient-to-b from-borda to-superficie p-4">
            <span className={`titulo text-texto ${grande ? "text-4xl" : "text-2xl"}`}>{nome}</span>
          </div>
        </>
      )}
    </div>
  )
}
