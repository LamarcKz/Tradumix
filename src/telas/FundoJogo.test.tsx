import { act, render, screen } from "@testing-library/react"
import { afterEach, beforeEach, expect, it, vi } from "vitest"

import { jogoPs2 } from "../testes/dados"
import type { Jogo } from "../tipos"
import { ESPERA_DO_TRAILER_MS, FundoJogo } from "./FundoJogo"

const jogo: Jogo = jogoPs2({ estado: "beta", trailer_youtube: "Pj-ueiUYEbg" }, {}, "data:image/png;base64,AAAA")
const fundo = { nome: jogo.nome, capa: jogo.plataformas[0].capa, traducao: jogo.plataformas[0].traducoes[0] }

function player() {
  return screen.getByTitle("Trailer de Jak II") as HTMLIFrameElement
}

/** O YouTube avisando o estado do player (1 = tocando, 0 = terminou). */
function estadoDoPlayer(estado: number) {
  act(() => {
    window.dispatchEvent(
      new MessageEvent("message", {
        origin: "https://www.youtube-nocookie.com",
        source: player().contentWindow,
        data: JSON.stringify({ event: "onStateChange", info: estado }),
      }),
    )
  })
}

beforeEach(() => vi.useFakeTimers())
afterEach(() => vi.useRealTimers())

it("trailer_sem_lista_de_reproducao_e_com_api", () => {
  render(<FundoJogo {...fundo} mostrarTrailers offline={false} />)
  const src = new URL(player().src)
  expect(src.searchParams.get("controls")).toBe("0")
  expect(src.searchParams.get("enablejsapi")).toBe("1")
  expect(src.searchParams.has("playlist")).toBe(false)
  expect(src.searchParams.has("loop")).toBe(false)
})

it("trailer_fica_escondido_ate_tocar_um_tempo", () => {
  render(<FundoJogo {...fundo} mostrarTrailers offline={false} />)
  expect(player()).toHaveClass("opacity-0")
  estadoDoPlayer(1)
  act(() => vi.advanceTimersByTime(ESPERA_DO_TRAILER_MS - 100))
  expect(player()).toHaveClass("opacity-0")
  act(() => vi.advanceTimersByTime(200))
  expect(player()).toHaveClass("opacity-80")
})

it("trailer_some_e_recomeca_quando_termina", () => {
  render(<FundoJogo {...fundo} mostrarTrailers offline={false} />)
  const enviar = vi.spyOn(player().contentWindow!, "postMessage")
  estadoDoPlayer(1)
  act(() => vi.advanceTimersByTime(ESPERA_DO_TRAILER_MS))
  estadoDoPlayer(0)
  expect(player()).toHaveClass("opacity-0")
  const comandos = enviar.mock.calls.map(([m]) => JSON.parse(String(m)).func)
  expect(comandos).toEqual(expect.arrayContaining(["seekTo", "playVideo"]))
})

it("mensagem_de_outra_janela_e_ignorada", () => {
  render(<FundoJogo {...fundo} mostrarTrailers offline={false} />)
  act(() => {
    window.dispatchEvent(new MessageEvent("message", { origin: "https://exemplo.org", data: JSON.stringify({ event: "onStateChange", info: 1 }) }))
    vi.advanceTimersByTime(ESPERA_DO_TRAILER_MS * 2)
  })
  expect(player()).toHaveClass("opacity-0")
})

it("aviso_repetido_de_tocando_nao_atrasa_o_trailer", () => {
  render(<FundoJogo {...fundo} mostrarTrailers offline={false} />)
  estadoDoPlayer(1)
  act(() => vi.advanceTimersByTime(ESPERA_DO_TRAILER_MS - 500))
  estadoDoPlayer(1)
  act(() => vi.advanceTimersByTime(600))
  expect(player()).toHaveClass("opacity-80")
})
