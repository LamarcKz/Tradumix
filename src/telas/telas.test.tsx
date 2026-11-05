import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, describe, expect, it, vi } from "vitest"

import * as api from "../api"
import type { Jogo, ListaJogos, Resultado } from "../tipos"
import { Inicio } from "./Inicio"
import { JogoTela } from "./Jogo"
import { ProgressoTela } from "./Progresso"
import { Pronto } from "./Pronto"

vi.mock("../api", () => ({
  listarJogos: vi.fn(),
  escolherPastaRascunhos: vi.fn(),
  conferirIso: vi.fn(),
  traduzir: vi.fn(),
  cancelarTraducao: vi.fn(),
  abrirPasta: vi.fn(),
  aoProgresso: vi.fn(() => () => {}),
  escolherArquivoIso: vi.fn(),
  escolherPasta: vi.fn(),
}))

const falso = vi.mocked(api)

const jak2: Jogo = {
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
      tradutores: [{ nome: "Fulano de Tal", link: "https://exemplo.org" }],
      permissao: null,
      descricao: "Menus, missões e legendas em português.",
      traduzido: ["Menus", "Legendas das cenas"],
      limitacoes: ["Algumas texturas continuam em inglês."],
      data: "2026-11-05",
      fundo: null,
    },
  ],
}

const lista = (parcial: Partial<ListaJogos> = {}): ListaJogos => ({
  jogos: [jak2, { ...jak2, id: "abc", nome: "Abc", traducoes: [{ ...jak2.traducoes[0], id: "abc-1" }] }],
  invalidos: [],
  atualizacoes: [],
  pasta_rascunhos: "C:/pacotes",
  ...parcial,
})

beforeEach(() => vi.clearAllMocks())

describe("Início", () => {
  it("inicio_mostra_um_card_por_jogo", async () => {
    const abrir = vi.fn()
    render(<Inicio lista={lista()} aoAbrirJogo={abrir} aoEscolherPasta={() => {}} />)
    const cards = screen.getAllByRole("button", { name: /Abrir/ })
    expect(cards).toHaveLength(2)
    await userEvent.click(screen.getByRole("button", { name: /Abrir Jak II/ }))
    expect(abrir).toHaveBeenCalledWith("jak2")
  })

  it("inicio_sem_pasta_mostra_estado_vazio", async () => {
    const escolher = vi.fn()
    render(<Inicio lista={lista({ jogos: [], pasta_rascunhos: null })} aoAbrirJogo={() => {}} aoEscolherPasta={escolher} />)
    await userEvent.click(screen.getByRole("button", { name: "Escolher pasta de rascunhos (modo de teste)" }))
    expect(escolher).toHaveBeenCalled()
  })

  it("inicio_mostra_pacotes_invalidos", () => {
    render(
      <Inicio
        lista={lista({ invalidos: [{ pasta: "C:/pacotes/quebrado", motivo: "ficha.json inválido" }] })}
        aoAbrirJogo={() => {}}
        aoEscolherPasta={() => {}}
      />,
    )
    expect(screen.getByText(/1 pacote com problema/)).toBeInTheDocument()
    expect(screen.getByText(/ficha.json inválido/)).toBeInTheDocument()
  })

  it("inicio_mostra_atualizacao_disponivel", () => {
    render(
      <Inicio
        lista={lista({ atualizacoes: [{ pacote_id: "jak2-usa-ptbr", versao_gerada: "1.0", versao_nova: "1.1" }] })}
        aoAbrirJogo={() => {}}
        aoEscolherPasta={() => {}}
      />,
    )
    expect(screen.getAllByText("Atualização disponível")).toHaveLength(1)
  })
})

describe("Jogo", () => {
  it("jogo_mostra_creditos_e_limitacoes", () => {
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.getByRole("heading", { name: "Jak II" })).toBeInTheDocument()
    expect(screen.getByRole("link", { name: "Fulano de Tal" })).toHaveAttribute("href", "https://exemplo.org")
    expect(screen.getByText("Algumas texturas continuam em inglês.")).toBeInTheDocument()
    expect(screen.getByText("Legendas das cenas")).toBeInTheDocument()
    expect(screen.getByText(/Rascunho/)).toBeInTheDocument()
  })

  it("iso_errada_mostra_mensagem_e_nao_traduz", async () => {
    const traduzir = vi.fn()
    falso.escolherArquivoIso.mockResolvedValue("C:/jogos/outro.iso")
    falso.conferirIso.mockResolvedValue({
      serve: false,
      mensagem: "Esta tradução precisa da Jak II (USA), serial SCUS-97265. A ISO escolhida é SLES-12345.",
      saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso",
      ja_existe: false,
    })
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={traduzir} />)
    await userEvent.click(screen.getByRole("button", { name: "Selecionar ISO" }))
    expect(await screen.findByText(/precisa da Jak II \(USA\)/)).toBeInTheDocument()
    expect(screen.queryByRole("button", { name: "Traduzir" })).not.toBeInTheDocument()
    expect(traduzir).not.toHaveBeenCalled()
  })

  it("iso_certa_mostra_saida_e_traduz", async () => {
    const traduzir = vi.fn()
    falso.escolherArquivoIso.mockResolvedValue("C:/jogos/jak2.iso")
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", ja_existe: false })
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={traduzir} />)
    await userEvent.click(screen.getByRole("button", { name: "Selecionar ISO" }))
    expect(await screen.findByText("C:/jogos/Jak II (USA) PT-BR v1.0.iso")).toBeInTheDocument()
    await userEvent.click(screen.getByRole("button", { name: "Traduzir" }))
    expect(traduzir).toHaveBeenCalledWith({
      pacoteId: "jak2-usa-ptbr",
      iso: "C:/jogos/jak2.iso",
      saida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso",
      sobrescrever: false,
    })
  })

  it("saida_existente_pede_confirmacao", async () => {
    const traduzir = vi.fn()
    falso.escolherArquivoIso.mockResolvedValue("C:/jogos/jak2.iso")
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", ja_existe: true })
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={traduzir} />)
    await userEvent.click(screen.getByRole("button", { name: "Selecionar ISO" }))
    expect(await screen.findByText("Já existe um arquivo com esse nome.")).toBeInTheDocument()
    expect(screen.queryByRole("button", { name: "Traduzir" })).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole("button", { name: "Substituir e traduzir" }))
    expect(traduzir).toHaveBeenCalledWith(expect.objectContaining({ sobrescrever: true }))
  })
})

describe("Progresso", () => {
  it("progresso_mostra_etapa_e_porcentagem", () => {
    render(<ProgressoTela nomeJogo="Jak II" progresso={{ etapa: "aplicando", fracao: 0.42 }} />)
    expect(screen.getByText("42%")).toBeInTheDocument()
    expect(screen.getByText("Aplicando a tradução").closest("li")).toHaveAttribute("aria-current", "step")
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "42")
  })

  it("cancelar_chama_api", async () => {
    render(<ProgressoTela nomeJogo="Jak II" progresso={{ etapa: "aplicando", fracao: 0.1 }} />)
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }))
    expect(falso.cancelarTraducao).toHaveBeenCalled()
  })
})

describe("Pronto", () => {
  const resultado = (compativel: boolean): Resultado => ({
    saida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso",
    hash_ra: "f51c94dd47cc2eb385d5733389428121",
    compativel_ra: compativel,
    segundos: 12.3,
  })

  it("pronto_mostra_compatibilidade_ra", async () => {
    render(<Pronto jogo={jak2} traducao={jak2.traducoes[0]} resultado={resultado(true)} aoVoltar={() => {}} />)
    expect(screen.getByText("Compatível com RetroAchievements")).toBeInTheDocument()
    expect(screen.getByText("Fulano de Tal")).toBeInTheDocument()
    await userEvent.click(screen.getByRole("button", { name: "Abrir pasta" }))
    await waitFor(() => expect(falso.abrirPasta).toHaveBeenCalledWith("C:/jogos/Jak II (USA) PT-BR v1.0.iso"))
  })

  it("pronto_avisa_quando_o_hash_mudou", () => {
    render(<Pronto jogo={jak2} traducao={jak2.traducoes[0]} resultado={resultado(false)} aoVoltar={() => {}} />)
    expect(screen.getByText(/as conquistas podem não funcionar/)).toBeInTheDocument()
  })
})
