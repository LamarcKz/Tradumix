import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, expect, it, vi } from "vitest"

import * as api from "./api"
import App from "./App"
import type { ListaJogos } from "./tipos"

vi.mock("./api", () => ({
  listarJogos: vi.fn(),
  escolherPastaRascunhos: vi.fn(),
  conferirIso: vi.fn(),
  traduzir: vi.fn(),
  cancelarTraducao: vi.fn(),
  abrirPasta: vi.fn(),
  aoProgresso: vi.fn(() => () => {}),
  escolherArquivoIso: vi.fn(),
  escolherPasta: vi.fn(),
  arquivoExiste: vi.fn(),
  minimizarJanela: vi.fn(),
  alternarMaximizarJanela: vi.fn(),
  fecharJanela: vi.fn(),
}))

const falso = vi.mocked(api)

const lista: ListaJogos = {
  jogos: [
    {
      id: "jak2",
      nome: "Jak II",
      console: "ps2",
      capa: null,
      traducoes: [
        {
          id: "jak2-usa-ptbr",
          versao: "1.0",
          estado: "rascunho",
          idioma: "pt-BR",
          regiao: "USA",
          serial: "SCUS-97265",
          tradutores: [{ nome: "Fulano", link: null }],
          permissao: null,
          descricao: "",
          traduzido: [],
          limitacoes: [],
          data: "2026-11-05",
          fundo: null,
        },
      ],
    },
  ],
  invalidos: [],
  atualizacoes: [],
  pasta_rascunhos: "C:/pacotes",
}

beforeEach(() => {
  vi.clearAllMocks()
  falso.listarJogos.mockResolvedValue(lista)
  falso.escolherArquivoIso.mockResolvedValue("C:/jogos/jak2.iso")
  falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", ja_existe: false })
})

async function ateTraduzir() {
  render(<App />)
  await userEvent.click(await screen.findByRole("button", { name: /Abrir Jak II/ }))
  await userEvent.click(screen.getByRole("button", { name: "Selecionar ISO" }))
  await userEvent.click(await screen.findByRole("button", { name: "Traduzir" }))
}

it("fluxo_completo_chega_na_tela_pronto", async () => {
  falso.traduzir.mockResolvedValue({ saida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", hash_ra: "x", compativel_ra: true, segundos: 3 })
  await ateTraduzir()
  expect(await screen.findByRole("heading", { name: "ISO traduzida pronta" })).toBeInTheDocument()
  expect(falso.traduzir).toHaveBeenCalledWith("jak2-usa-ptbr", "C:/jogos/jak2.iso", "C:/jogos/Jak II (USA) PT-BR v1.0.iso", false)
})

it("erro_na_traducao_volta_ao_jogo_com_a_mensagem", async () => {
  falso.traduzir.mockRejectedValue("Tradução cancelada.")
  await ateTraduzir()
  expect(await screen.findByText("Tradução cancelada.")).toBeInTheDocument()
  expect(screen.getByRole("heading", { name: "Jak II" })).toBeInTheDocument()
  expect(await screen.findByRole("button", { name: "Traduzir" })).toBeInTheDocument()
  expect(screen.getByText("C:/jogos/jak2.iso")).toBeInTheDocument()
})

it("topo_tem_os_botoes_da_janela", async () => {
  render(<App />)
  await userEvent.click(await screen.findByRole("button", { name: "Minimizar" }))
  await userEvent.click(screen.getByRole("button", { name: "Maximizar" }))
  await userEvent.click(screen.getByRole("button", { name: "Fechar" }))
  expect(falso.minimizarJanela).toHaveBeenCalled()
  expect(falso.alternarMaximizarJanela).toHaveBeenCalled()
  expect(falso.fecharJanela).toHaveBeenCalled()
  expect(document.querySelector("header")).toHaveAttribute("data-tauri-drag-region")
})

it("configuracoes_troca_a_pasta_de_rascunhos", async () => {
  falso.escolherPasta.mockResolvedValue("D:/outra")
  falso.escolherPastaRascunhos.mockResolvedValue({ ...lista, pasta_rascunhos: "D:/outra" })
  render(<App />)
  await userEvent.click(await screen.findByRole("button", { name: "Configurações" }))
  await userEvent.click(screen.getByRole("button", { name: "Trocar pasta" }))
  expect(falso.escolherPastaRascunhos).toHaveBeenCalledWith("D:/outra")
  expect(await screen.findByText("D:/outra")).toBeInTheDocument()
})
