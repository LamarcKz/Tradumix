import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, expect, it, vi } from "vitest"

import * as api from "./api"
import App from "./App"
import type { ListaJogos } from "./tipos"

vi.mock("./api", () => ({
  listarJogos: vi.fn(),
  sincronizarCatalogo: vi.fn(),
  escolherPastaRascunhos: vi.fn(),
  definirMostrarTrailers: vi.fn(),
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
  procurarVersaoNova: vi.fn(() => Promise.resolve(null)),
  instalarVersaoNova: vi.fn(),
  versaoDoApp: vi.fn(() => Promise.resolve({ versao: "0.1.0", portatil: false })),
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
          origem: "catalogo",
          novo: false,
          patch_disponivel: true,
          tamanho_download: null,
          trailer_youtube: null,
        },
      ],
    },
  ],
  invalidos: [],
  atualizacoes: [],
  pasta_rascunhos: "C:/pacotes",
  catalogo: { offline: false, atualizado_em: "2026-11-05", aviso: null },
  mostrar_trailers: true,
}

beforeEach(() => {
  vi.clearAllMocks()
  falso.listarJogos.mockResolvedValue(lista)
  falso.sincronizarCatalogo.mockResolvedValue(lista)
  falso.escolherArquivoIso.mockResolvedValue("C:/jogos/jak2.iso")
  falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", ja_existe: false })
})

async function ateTraduzir() {
  render(<App />)
  await userEvent.click(await screen.findByRole("button", { name: /Abrir Jak II/ }))
  await userEvent.click(screen.getByRole("button", { name: "Selecionar ISO" }))
  await userEvent.click(await screen.findByRole("button", { name: "Traduzir" }))
}

it("app_sincroniza_ao_abrir", async () => {
  const atualizada = { ...lista, jogos: [...lista.jogos, { ...lista.jogos[0], id: "sly", nome: "Sly Cooper" }] }
  falso.sincronizarCatalogo.mockResolvedValue(atualizada)
  render(<App />)
  expect(await screen.findByRole("button", { name: /Abrir Sly Cooper/ })).toBeInTheDocument()
  expect(falso.listarJogos).toHaveBeenCalled()
  expect(falso.sincronizarCatalogo).toHaveBeenCalledTimes(1)
})

it("jogo_que_some_do_catalogo_volta_ao_inicio", async () => {
  let liberar: (l: ListaJogos) => void = () => {}
  falso.sincronizarCatalogo.mockReturnValue(new Promise((r) => (liberar = r)))
  render(<App />)
  await userEvent.click(await screen.findByRole("button", { name: /Abrir Jak II/ }))
  expect(screen.getByRole("heading", { name: "Jak II" })).toBeInTheDocument()
  liberar({ ...lista, jogos: [] })
  expect(await screen.findByText("Nenhuma tradução disponível ainda.")).toBeInTheDocument()
})

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

it("configuracoes_desliga_o_trailer", async () => {
  falso.definirMostrarTrailers.mockResolvedValue({ ...lista, mostrar_trailers: false })
  render(<App />)
  await userEvent.click(await screen.findByRole("button", { name: "Configurações" }))
  const opcao = screen.getByRole("checkbox", { name: "Mostrar o trailer atrás do título" })
  expect(opcao).toBeChecked()
  await userEvent.click(opcao)
  expect(falso.definirMostrarTrailers).toHaveBeenCalledWith(false)
  expect(await screen.findByRole("checkbox", { name: "Mostrar o trailer atrás do título" })).not.toBeChecked()
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

it("topo_abre_a_tela_sobre", async () => {
  render(<App />)
  await userEvent.click(await screen.findByRole("button", { name: "Sobre" }))
  expect(await screen.findByText("Versão 0.1.0")).toBeInTheDocument()
  expect(screen.queryByRole("button", { name: "Sobre" })).not.toBeInTheDocument()
})
