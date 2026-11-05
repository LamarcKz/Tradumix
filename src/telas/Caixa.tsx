import type { Jogo } from "../tipos"

/**
 * A capa no formato de caixa de DVD de PS2. As capas do acervo já trazem a faixa preta do PS2;
 * sem capa, o app desenha a faixa e o nome do jogo.
 */
export function Caixa({ jogo, tamanho = "normal" }: { jogo: Jogo; tamanho?: "normal" | "grande" }) {
  const grande = tamanho === "grande"
  return (
    <div
      className={`relative aspect-[7/10] overflow-hidden rounded-[6px] bg-caixa shadow-[0_18px_40px_-18px_rgba(0,0,0,0.8)] ring-1 ring-white/10 ${
        grande ? "w-64" : "w-full"
      }`}
    >
      {jogo.capa ? (
        <img src={jogo.capa} alt="" className="h-full w-full object-cover" />
      ) : (
        <>
          <div className={`flex items-center bg-caixa px-3 ${grande ? "h-9" : "h-7"}`}>
            <span className={`font-semibold tracking-tight text-white/90 ${grande ? "text-sm" : "text-[11px]"}`}>PlayStation 2</span>
          </div>
          <div className="flex h-[calc(100%-1.75rem)] items-end bg-gradient-to-b from-borda to-superficie p-4">
            <span className={`titulo text-texto ${grande ? "text-4xl" : "text-2xl"}`}>{jogo.nome}</span>
          </div>
        </>
      )}
    </div>
  )
}
