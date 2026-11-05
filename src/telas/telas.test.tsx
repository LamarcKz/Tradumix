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
  sincronizarCatalogo: vi.fn(),
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
      origem: "catalogo",
      novo: false,
      patch_disponivel: true,
      tamanho_download: null,
    },
  ],
}

const lista = (parcial: Partial<ListaJogos> = {}): ListaJogos => ({
  jogos: [jak2, { ...jak2, id: "abc", nome: "Abc", traducoes: [{ ...jak2.traducoes[0], id: "abc-1" }] }],
  invalidos: [],
  atualizacoes: [],
  pasta_rascunhos: null,
  catalogo: { offline: false, atualizado_em: "2026-11-05", aviso: null },
  ...parcial,
})

const comTraducao = (extra: Partial<Jogo["traducoes"][number]>): Jogo => ({ ...jak2, traducoes: [{ ...jak2.traducoes[0], ...extra }] })

beforeEach(() => vi.clearAllMocks())

describe("Início", () => {
  it("inicio_mostra_um_card_por_jogo", async () => {
    const abrir = vi.fn()
    render(<Inicio lista={lista()} aoAbrirJogo={abrir} aoTentarDeNovo={() => {}} />)
    const cards = screen.getAllByRole("button", { name: /Abrir/ })
    expect(cards).toHaveLength(2)
    await userEvent.click(screen.getByRole("button", { name: /Abrir Jak II/ }))
    expect(abrir).toHaveBeenCalledWith("jak2")
  })

  it("inicio_vazio_tem_tentar_de_novo", async () => {
    const tentar = vi.fn()
    render(<Inicio lista={lista({ jogos: [] })} aoAbrirJogo={() => {}} aoTentarDeNovo={tentar} />)
    expect(screen.getByText("Nenhuma tradução disponível ainda.")).toBeInTheDocument()
    await userEvent.click(screen.getByRole("button", { name: "Tentar de novo" }))
    expect(tentar).toHaveBeenCalled()
  })

  it("inicio_nao_pede_pasta", () => {
    render(<Inicio lista={lista({ jogos: [] })} aoAbrirJogo={() => {}} aoTentarDeNovo={() => {}} />)
    expect(screen.queryByText(/pasta/i)).not.toBeInTheDocument()
  })

  it("inicio_mostra_selo_novo_e_rascunho", () => {
    const novo = { ...comTraducao({ novo: true }), id: "jak2" }
    const rascunho = { ...comTraducao({ origem: "rascunho", id: "rascunho:abc" }), id: "abc", nome: "Abc" }
    render(<Inicio lista={lista({ jogos: [novo, rascunho] })} aoAbrirJogo={() => {}} aoTentarDeNovo={() => {}} />)
    expect(screen.getAllByText("Novo")).toHaveLength(1)
    expect(screen.getAllByText("Rascunho")).toHaveLength(1)
  })

  it("inicio_offline_mostra_aviso_com_data", () => {
    const aviso = "Sem conexão: mostrando o catálogo baixado em 05/11/2026."
    render(
      <Inicio
        lista={lista({ catalogo: { offline: true, atualizado_em: "2026-11-05", aviso } })}
        aoAbrirJogo={() => {}}
        aoTentarDeNovo={() => {}}
      />,
    )
    expect(screen.getByText(aviso)).toBeInTheDocument()
  })

  it("inicio_mostra_que_esta_atualizando", () => {
    render(<Inicio lista={lista()} sincronizando aoAbrirJogo={() => {}} aoTentarDeNovo={() => {}} />)
    expect(screen.getByText("Atualizando o catálogo…")).toBeInTheDocument()
  })

  it("inicio_mostra_pacotes_invalidos", () => {
    render(
      <Inicio
        lista={lista({ invalidos: [{ pasta: "C:/pacotes/quebrado", motivo: "ficha.json inválido" }] })}
        aoAbrirJogo={() => {}}
        aoTentarDeNovo={() => {}}
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
        aoTentarDeNovo={() => {}}
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

describe("Jogo, outra pasta", () => {
  it("escolher_outra_pasta_confere_se_o_arquivo_existe", async () => {
    const traduzir = vi.fn()
    falso.escolherArquivoIso.mockResolvedValue("C:/jogos/jak2.iso")
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", ja_existe: false })
    falso.escolherPasta.mockResolvedValue("D:/outra")
    falso.arquivoExiste.mockResolvedValue(true)
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={traduzir} />)
    await userEvent.click(screen.getByRole("button", { name: "Selecionar ISO" }))
    await userEvent.click(await screen.findByRole("button", { name: "Escolher outra pasta" }))
    expect(falso.arquivoExiste).toHaveBeenCalledWith("D:/outra\\Jak II (USA) PT-BR v1.0.iso")
    await userEvent.click(await screen.findByRole("button", { name: "Substituir e traduzir" }))
    expect(traduzir).toHaveBeenCalledWith(expect.objectContaining({ saida: "D:/outra\\Jak II (USA) PT-BR v1.0.iso", sobrescrever: true }))
  })

  it("volta_de_um_erro_com_a_iso_e_a_saida_escolhidas", async () => {
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", ja_existe: false })
    falso.arquivoExiste.mockResolvedValue(true)
    render(
      <JogoTela
        jogo={jak2}
        aviso="Já existe um arquivo em D:/outra."
        pedidoAnterior={{ pacoteId: "jak2-usa-ptbr", iso: "C:/jogos/jak2.iso", saida: "D:/outra\\Jak II (USA) PT-BR v1.0.iso", sobrescrever: false }}
        aoVoltar={() => {}}
        aoTraduzir={() => {}}
      />,
    )
    expect(await screen.findByText("D:/outra\\Jak II (USA) PT-BR v1.0.iso")).toBeInTheDocument()
    expect(await screen.findByRole("button", { name: "Substituir e traduzir" })).toBeInTheDocument()
    expect(screen.getByText("C:/jogos/jak2.iso")).toBeInTheDocument()
  })
})

describe("Jogo, catálogo", () => {
  it("jogo_offline_sem_patch_nao_traduz", () => {
    render(<JogoTela jogo={comTraducao({ patch_disponivel: false, tamanho_download: 238940 })} offline aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.getByText("Precisa de internet para baixar a tradução.")).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Selecionar ISO" })).toBeDisabled()
  })

  it("jogo_mostra_tamanho_do_download", () => {
    render(<JogoTela jogo={comTraducao({ patch_disponivel: false, tamanho_download: 238940 })} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.getByText("Baixa 233 KB da internet na hora de traduzir.")).toBeInTheDocument()
  })
})

describe("Progresso", () => {
  it("progresso_mostra_etapa_baixando", () => {
    render(<ProgressoTela nomeJogo="Jak II" comDownload progresso={{ etapa: "baixando", fracao: 0.5 }} />)
    expect(screen.getByText("Baixando a tradução").closest("li")).toHaveAttribute("aria-current", "step")
    expect(screen.getByText("Etapa 2 de 4")).toBeInTheDocument()
  })

  it("progresso_sem_download_nao_mostra_baixando", () => {
    render(<ProgressoTela nomeJogo="Jak II" progresso={{ etapa: "aplicando", fracao: 0.5 }} />)
    expect(screen.queryByText("Baixando a tradução")).not.toBeInTheDocument()
  })

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
