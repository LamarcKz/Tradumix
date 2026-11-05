import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, expect, it, vi } from "vitest"

import * as api from "../api"
import { Sobre } from "./Sobre"

vi.mock("../api", () => ({
  versaoDoApp: vi.fn(),
}))

const falso = vi.mocked(api)

beforeEach(() => {
  vi.resetAllMocks()
  falso.versaoDoApp.mockResolvedValue({ versao: "0.1.0", portatil: false })
})

it("sobre_mostra_versao_e_licencas", async () => {
  render(<Sobre aoVoltar={() => {}} />)
  expect(await screen.findByText("Versão 0.1.0")).toBeInTheDocument()
  expect(screen.getByText("Tradumix — MIT")).toBeInTheDocument()
  expect(screen.getByText("xdelta3 3.2.2 — Apache 2.0, Joshua MacDonald")).toBeInTheDocument()
  expect(screen.getByText("Capas: acervo ps2-covers, usado pelo PCSX2")).toBeInTheDocument()
  expect(screen.getByText("Trailers: player do YouTube")).toBeInTheDocument()
})

it("sobre_mostra_que_e_portatil", async () => {
  falso.versaoDoApp.mockResolvedValue({ versao: "0.1.0", portatil: true })
  render(<Sobre aoVoltar={() => {}} />)
  expect(await screen.findByText("Versão 0.1.0 (portátil)")).toBeInTheDocument()
})

it("sobre_tem_links", async () => {
  render(<Sobre aoVoltar={() => {}} />)
  await screen.findByText("Versão 0.1.0")
  expect(screen.getByRole("link", { name: "Código do Tradumix" })).toHaveAttribute("href", "https://github.com/LamarcKz/Tradumix")
  expect(screen.getByRole("link", { name: "Catálogo de traduções" })).toHaveAttribute("href", "https://github.com/LamarcKz/Tradumix-catalogo")
  expect(screen.getByRole("link", { name: "Código do xdelta3" })).toHaveAttribute("href", "https://github.com/jmacd/xdelta")
  expect(screen.getByRole("link", { name: "Acervo ps2-covers" })).toHaveAttribute("href", "https://github.com/xlenore/ps2-covers")
})

it("sobre_volta", async () => {
  const aoVoltar = vi.fn()
  render(<Sobre aoVoltar={aoVoltar} />)
  await userEvent.click(screen.getByRole("button", { name: "← Traduções" }))
  expect(aoVoltar).toHaveBeenCalled()
})
