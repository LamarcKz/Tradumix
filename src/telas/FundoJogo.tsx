import { useEffect, useRef, useState } from "react"

import type { Jogo, Traducao } from "../tipos"

interface Props {
  jogo: Jogo
  traducao: Traducao
  mostrarTrailers: boolean
  offline: boolean
}

/**
 * Quanto o trailer toca escondido antes de aparecer. Nos primeiros segundos (e a cada recomeço) o
 * YouTube mostra por cima do vídeo o título, o logo e os botões, mesmo com `controls=0`.
 */
export const ESPERA_DO_TRAILER_MS = 3500

const ORIGEM_DO_PLAYER = "https://www.youtube-nocookie.com"

/**
 * O fundo da página do jogo: a capa ampliada e desfocada e, por cima, o trailer no player do próprio
 * YouTube, sem som. O app controla o player pela API do iframe (postMessage): mostra o vídeo só
 * depois de um tempo tocando e, quando termina, esconde, volta ao começo e toca de novo. Assim não
 * usa a lista de reprodução do YouTube (que traz os botões de anterior e próximo).
 */
export function FundoJogo({ jogo, traducao, mostrarTrailers, offline }: Props) {
  const trailer = mostrarTrailers && !offline ? traducao.trailer_youtube : null
  const imagem = traducao.fundo ?? jogo.capa
  const player = useRef<HTMLIFrameElement>(null)
  const [visivel, setVisivel] = useState(false)

  useEffect(() => {
    if (!trailer) return
    let espera: ReturnType<typeof setTimeout> | undefined
    let ultimo: number | undefined
    const comando = (func: string, args: unknown[] = []) =>
      player.current?.contentWindow?.postMessage(JSON.stringify({ event: "command", func, args }), ORIGEM_DO_PLAYER)

    function aoReceber(e: MessageEvent) {
      if (e.origin !== ORIGEM_DO_PLAYER || e.source !== player.current?.contentWindow) return
      let dados: { event?: string; info?: unknown }
      try {
        dados = typeof e.data === "string" ? JSON.parse(e.data) : e.data
      } catch {
        return
      }
      const estado =
        dados.event === "onStateChange" ? dados.info : (dados.info as { playerState?: number } | undefined)?.playerState
      // O YouTube pode repetir o mesmo estado; só a mudança conta.
      if (typeof estado !== "number" || estado === ultimo) return
      ultimo = estado
      if (estado === 1) {
        clearTimeout(espera)
        espera = setTimeout(() => setVisivel(true), ESPERA_DO_TRAILER_MS)
      } else if (estado === 0) {
        clearTimeout(espera)
        setVisivel(false)
        comando("seekTo", [0, true])
        comando("playVideo")
      }
    }

    window.addEventListener("message", aoReceber)
    return () => {
      clearTimeout(espera)
      window.removeEventListener("message", aoReceber)
    }
  }, [trailer])

  if (!imagem && !trailer) return null

  const parametros = new URLSearchParams({
    autoplay: "1",
    mute: "1",
    controls: "0",
    disablekb: "1",
    modestbranding: "1",
    playsinline: "1",
    rel: "0",
    iv_load_policy: "3",
    enablejsapi: "1",
    origin: window.location.origin,
  })

  /** Pede ao player que avise as mudanças de estado (é assim que a API do iframe começa). */
  function aoCarregar() {
    player.current?.contentWindow?.postMessage(JSON.stringify({ event: "listening", id: 1, channel: "widget" }), ORIGEM_DO_PLAYER)
  }

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
          ref={player}
          title={`Trailer de ${jogo.nome}`}
          src={`${ORIGEM_DO_PLAYER}/embed/${trailer}?${parametros}`}
          allow="autoplay; encrypted-media"
          referrerPolicy="strict-origin-when-cross-origin"
          tabIndex={-1}
          onLoad={aoCarregar}
          className={`absolute left-1/2 top-1/2 aspect-video w-[160%] min-w-[1400px] -translate-x-1/2 -translate-y-1/2 border-0 transition-opacity duration-1000 ${visivel ? "opacity-80" : "opacity-0"}`}
        />
      )}
      {/* Topo quase limpo para o vídeo; escurece onde começam o título e o texto e some no fundo do app. */}
      <div className="absolute inset-0 bg-gradient-to-b from-transparent from-25% via-fundo/70 via-65% to-fundo" />
      <div className="absolute inset-0 bg-gradient-to-r from-fundo/60 via-transparent via-40% to-transparent" />
    </div>
  )
}
