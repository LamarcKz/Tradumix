import type { Jogo, Traducao } from "../tipos"

interface Props {
  jogo: Jogo
  traducao: Traducao
  mostrarTrailers: boolean
  offline: boolean
}

/**
 * O fundo da página do jogo: a arte do pacote (rascunhos) ou a capa ampliada e desfocada e, por cima,
 * o trailer no player do próprio YouTube, sem som e em loop. Nada disso fica guardado no catálogo.
 */
export function FundoJogo({ jogo, traducao, mostrarTrailers, offline }: Props) {
  const trailer = mostrarTrailers && !offline ? traducao.trailer_youtube : null
  const imagem = traducao.fundo ?? jogo.capa
  if (!imagem && !trailer) return null

  const parametros = new URLSearchParams({
    autoplay: "1",
    mute: "1",
    loop: "1",
    playlist: trailer ?? "",
    controls: "0",
    disablekb: "1",
    modestbranding: "1",
    playsinline: "1",
    rel: "0",
    iv_load_policy: "3",
  })

  return (
    <div
      aria-hidden
      className={`pointer-events-none absolute inset-x-0 top-0 overflow-hidden ${trailer ? "h-[82vh] min-h-[520px]" : "h-[560px]"}`}
    >
      {imagem &&
        (traducao.fundo ? (
          <img src={imagem} alt="" className="h-full w-full object-cover opacity-45" />
        ) : (
          <img data-fundo="capa" src={imagem} alt="" className="h-full w-full scale-125 object-cover opacity-40 blur-2xl" />
        ))}
      {trailer && (
        <iframe
          title={`Trailer de ${jogo.nome}`}
          src={`https://www.youtube-nocookie.com/embed/${trailer}?${parametros}`}
          allow="autoplay; encrypted-media"
          referrerPolicy="strict-origin-when-cross-origin"
          tabIndex={-1}
          className="absolute left-1/2 top-1/2 aspect-video w-[160%] min-w-[1400px] -translate-x-1/2 -translate-y-1/2 border-0 opacity-80"
        />
      )}
      {/* Topo quase limpo para o vídeo; escurece onde começam o título e o texto e some no fundo do app. */}
      <div className="absolute inset-0 bg-gradient-to-b from-transparent from-25% via-fundo/70 via-65% to-fundo" />
      <div className="absolute inset-0 bg-gradient-to-r from-fundo/60 via-transparent via-40% to-transparent" />
    </div>
  )
}
