import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, expect, it, vi } from "vitest"

import * as api from "../api"
import { AvisoVersao } from "./AvisoVersao"

vi.mock("../api", () => ({
  procurarVersaoNova: vi.fn(),
  instalarVersaoNova: vi.fn(),
}))

const falso = vi.mocked(api)

beforeEach(() => {
  vi.resetAllMocks()
  falso.instalarVersaoNova.mockResolvedValue()
})

it("mostra_aviso_quando_ha_versao_nova", async () => {
  falso.procurarVersaoNova.mockResolvedValue({ versao: "0.2.0", notas: "Tela Sobre nova.", portatil: false })
  render(<AvisoVersao />)
  expect(await screen.findByText("Versão 0.2.0 disponível")).toBeInTheDocument()
  expect(screen.getByRole("button", { name: "Atualizar" })).toBeInTheDocument()
  await userEvent.click(screen.getByRole("button", { name: "Ver novidades" }))
  expect(screen.getByText("Tela Sobre nova.")).toBeInTheDocument()
})

it("sem_versao_nova_nao_mostra_nada", async () => {
  falso.procurarVersaoNova.mockResolvedValue(null)
  const { container } = render(<AvisoVersao />)
  await waitFor(() => expect(falso.procurarVersaoNova).toHaveBeenCalledTimes(1))
  expect(container).toBeEmptyDOMElement()
})

it("erro_ao_procurar_nao_mostra_nada", async () => {
  falso.procurarVersaoNova.mockRejectedValue(new Error("sem internet"))
  const { container } = render(<AvisoVersao />)
  await waitFor(() => expect(falso.procurarVersaoNova).toHaveBeenCalledTimes(1))
  expect(container).toBeEmptyDOMElement()
})

it("atualizar_chama_a_api", async () => {
  falso.procurarVersaoNova.mockResolvedValue({ versao: "0.2.0", notas: "", portatil: false })
  render(<AvisoVersao />)
  await userEvent.click(await screen.findByRole("button", { name: "Atualizar" }))
  expect(falso.instalarVersaoNova).toHaveBeenCalledTimes(1)
  expect(screen.getByText("Instalando a versão 0.2.0… o Tradumix reinicia sozinho.")).toBeInTheDocument()
})

it("portatil_abre_a_pagina_de_download", async () => {
  falso.procurarVersaoNova.mockResolvedValue({ versao: "0.2.0", notas: "", portatil: true })
  render(<AvisoVersao />)
  await userEvent.click(await screen.findByRole("button", { name: "Baixar" }))
  expect(falso.instalarVersaoNova).toHaveBeenCalledTimes(1)
})

it("erro_ao_atualizar_explica", async () => {
  falso.procurarVersaoNova.mockResolvedValue({ versao: "0.2.0", notas: "", portatil: false })
  falso.instalarVersaoNova.mockRejectedValue("sem internet")
  render(<AvisoVersao />)
  await userEvent.click(await screen.findByRole("button", { name: "Atualizar" }))
  expect(await screen.findByText("Não deu para atualizar: sem internet")).toBeInTheDocument()
  expect(screen.getByRole("button", { name: "Atualizar" })).toBeEnabled()
})

it("fechar_o_aviso_esconde_ate_a_proxima_abertura", async () => {
  falso.procurarVersaoNova.mockResolvedValue({ versao: "0.2.0", notas: "", portatil: false })
  const { container } = render(<AvisoVersao />)
  await userEvent.click(await screen.findByRole("button", { name: "Fechar aviso" }))
  expect(container).toBeEmptyDOMElement()
})
