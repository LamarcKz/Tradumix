import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, describe, expect, it, vi } from "vitest"

import * as api from "../api"
import { infoPc, infoPs1, infoPs2, jogoPs2, jogoPs2ePc, listaDe, plataforma, traducaoJak2 } from "../testes/dados"
import type { Jogo, ListaJogos, Resultado, Traducao } from "../tipos"
import { Caixa } from "./Caixa"
import { Inicio } from "./Inicio"
import { JogoTela } from "./Jogo"
import { ProgressoTela } from "./Progresso"
import { Pronto } from "./Pronto"

vi.mock("../api", () => ({
  listarJogos: vi.fn(),
  sincronizarCatalogo: vi.fn(),
  definirMostrarTrailers: vi.fn(),
  conferirIso: vi.fn(),
  traduzir: vi.fn(),
  removerTraducao: vi.fn(),
  cancelarTraducao: vi.fn(),
  abrirPasta: vi.fn(),
  aoProgresso: vi.fn(() => () => {}),
  escolherArquivoDoJogo: vi.fn(),
  escolherPasta: vi.fn(),
  arquivoExiste: vi.fn(),
  minimizarJanela: vi.fn(),
  alternarMaximizarJanela: vi.fn(),
  fecharJanela: vi.fn(),
}))

const falso = vi.mocked(api)

const jak2: Jogo = jogoPs2()
const traducao = jak2.plataformas[0].traducoes[0]

const lista = (parcial: Partial<ListaJogos> = {}): ListaJogos =>
  listaDe([jak2, jogoPs2({ id: "abc-1" }, { id: "abc", nome: "Abc" })], parcial)

const comTraducao = (extra: Partial<Traducao>): Jogo => jogoPs2(extra)

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

  it("inicio_mostra_so_o_selo_novo", () => {
    const novo = { ...comTraducao({ novo: true, estado: "beta" }), id: "jak2" }
    const outro = { ...comTraducao({ estado: "rascunho" }), id: "abc", nome: "Abc" }
    render(<Inicio lista={lista({ jogos: [novo, outro] })} aoAbrirJogo={() => {}} aoTentarDeNovo={() => {}} />)
    expect(screen.getAllByText("Novo")).toHaveLength(1)
    expect(screen.queryByText("Beta")).not.toBeInTheDocument()
    expect(screen.queryByText("Rascunho")).not.toBeInTheDocument()
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

  it("inicio_mostra_a_plataforma_no_card", () => {
    render(<Inicio lista={listaDe([jak2])} aoAbrirJogo={() => {}} aoTentarDeNovo={() => {}} />)
    expect(screen.getByText("PS2")).toBeInTheDocument()
  })

  it("inicio_sem_filtro_com_uma_plataforma", () => {
    render(<Inicio lista={lista()} aoAbrirJogo={() => {}} aoTentarDeNovo={() => {}} />)
    expect(screen.queryByRole("button", { name: "Todas" })).not.toBeInTheDocument()
  })

  it("inicio_filtra_por_plataforma", async () => {
    const abrir = vi.fn()
    const a = jogoPs2({ id: "a-1" }, { id: "a", nome: "Abc" })
    const b = { ...jogoPs2ePc(), id: "b", nome: "Bcd" }
    const c: Jogo = { id: "c", nome: "Cde", plataformas: [plataforma(infoPc, [{ ...traducaoJak2, id: "c-1" }])] }
    render(<Inicio lista={listaDe([a, b, c])} aoAbrirJogo={abrir} aoTentarDeNovo={() => {}} />)
    expect(screen.getByText("PS2 · PC")).toBeInTheDocument()
    expect(screen.getAllByRole("button", { name: /Abrir/ })).toHaveLength(3)
    await userEvent.click(screen.getByRole("button", { name: "PC" }))
    const nomes = screen.getAllByRole("button", { name: /Abrir/ }).map((b) => b.getAttribute("aria-label"))
    expect(nomes).toEqual(["Abrir Bcd", "Abrir Cde"])
    await userEvent.click(screen.getByRole("button", { name: "Abrir Bcd" }))
    expect(abrir).toHaveBeenCalledWith("b", "pc")
    await userEvent.click(screen.getByRole("button", { name: "Todas" }))
    expect(screen.getAllByRole("button", { name: /Abrir/ })).toHaveLength(3)
  })

  it("inicio_filtro_usa_a_capa_da_plataforma", async () => {
    const b = jogoPs2ePc()
    b.plataformas[0].capa = "data:image/png;base64,PS2"
    b.plataformas[1].capa = "data:image/png;base64,PC"
    render(<Inicio lista={listaDe([b])} aoAbrirJogo={() => {}} aoTentarDeNovo={() => {}} />)
    expect(document.querySelector("img")).toHaveAttribute("src", "data:image/png;base64,PS2")
    await userEvent.click(screen.getByRole("button", { name: "PC" }))
    expect(document.querySelector("img")).toHaveAttribute("src", "data:image/png;base64,PC")
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
  it("jogo_mostra_o_que_foi_e_o_que_nao_foi_traduzido", () => {
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.getByRole("heading", { name: "Jak II" })).toBeInTheDocument()
    expect(screen.getByRole("heading", { name: "O que foi traduzido" })).toBeInTheDocument()
    expect(screen.getByText("Legendas das cenas")).toBeInTheDocument()
    expect(screen.getByRole("heading", { name: "O que não foi traduzido" })).toBeInTheDocument()
    expect(screen.getByText("Algumas texturas continuam em inglês.")).toBeInTheDocument()
    expect(screen.queryByText("Limitações")).not.toBeInTheDocument()
    expect(screen.queryByText("Menus, missões e legendas em português.")).not.toBeInTheDocument()
    expect(screen.queryByText(/Rascunho|Beta/)).not.toBeInTheDocument()
  })

  it("jogo_creditos_com_nick_e_github", () => {
    const comGithub = comTraducao({ tradutores: [{ nome: "LamarcKz", link: "https://github.com/LamarcKz" }] })
    render(<JogoTela jogo={comGithub} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.getByText("LamarcKz")).toBeInTheDocument()
    expect(screen.getByRole("link", { name: "github.com/LamarcKz" })).toHaveAttribute("href", "https://github.com/LamarcKz")
  })

  it("jogo_creditos_com_outro_link", () => {
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.getByText("Fulano de Tal")).toBeInTheDocument()
    expect(screen.getByRole("link", { name: "exemplo.org" })).toHaveAttribute("href", "https://exemplo.org")
  })

  it("iso_errada_mostra_mensagem_e_nao_traduz", async () => {
    const traduzir = vi.fn()
    falso.escolherArquivoDoJogo.mockResolvedValue("C:/jogos/outro.iso")
    falso.conferirIso.mockResolvedValue({
      serve: false,
      mensagem: "Esta tradução precisa da Jak II (USA), serial SCUS-97265. A ISO escolhida é SLES-12345.",
      aviso: null,
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
    falso.escolherArquivoDoJogo.mockResolvedValue("C:/jogos/jak2.iso")
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, aviso: null, saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", ja_existe: false })
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={traduzir} />)
    await userEvent.click(screen.getByRole("button", { name: "Selecionar ISO" }))
    expect(await screen.findByText("C:/jogos/Jak II (USA) PT-BR v1.0.iso")).toBeInTheDocument()
    await userEvent.click(screen.getByRole("button", { name: "Traduzir" }))
    expect(traduzir).toHaveBeenCalledWith({
      pacoteId: "jak2-usa-ptbr",
      plataformaId: "ps2",
      iso: "C:/jogos/jak2.iso",
      saida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso",
      sobrescrever: false,
    })
  })

  it("saida_existente_pede_confirmacao", async () => {
    const traduzir = vi.fn()
    falso.escolherArquivoDoJogo.mockResolvedValue("C:/jogos/jak2.iso")
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, aviso: null, saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", ja_existe: true })
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={traduzir} />)
    await userEvent.click(screen.getByRole("button", { name: "Selecionar ISO" }))
    expect(await screen.findByText("Já existe um arquivo com esse nome.")).toBeInTheDocument()
    expect(screen.queryByRole("button", { name: "Traduzir" })).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole("button", { name: "Substituir e traduzir" }))
    expect(traduzir).toHaveBeenCalledWith(expect.objectContaining({ sobrescrever: true }))
  })
})

describe("Jogo, plataformas", () => {
  it("jogo_com_uma_plataforma_nao_mostra_abas", () => {
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.queryByRole("tab")).not.toBeInTheDocument()
    expect(screen.getByText("Tradução PT-BR versão 1.0, para a versão USA (SCUS-97265)")).toBeInTheDocument()
  })

  it("jogo_com_duas_plataformas_mostra_abas_e_troca_as_traducoes", async () => {
    render(<JogoTela jogo={jogoPs2ePc()} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.getByRole("tab", { name: "PlayStation 2" })).toHaveAttribute("aria-selected", "true")
    expect(screen.getByRole("button", { name: "Selecionar ISO" })).toBeInTheDocument()
    await userEvent.click(screen.getByRole("tab", { name: "PC" }))
    expect(screen.getByRole("tab", { name: "PC" })).toHaveAttribute("aria-selected", "true")
    expect(screen.getByText("Tradução PT-BR versão 2.0, para a versão Steam")).toBeInTheDocument()
    expect(screen.getByText("Escolha a pasta do jogo.")).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Escolher pasta do jogo" })).toBeInTheDocument()
  })

  it("abre_na_aba_da_plataforma_do_filtro", () => {
    render(<JogoTela jogo={jogoPs2ePc()} plataformaInicial="pc" aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.getByRole("tab", { name: "PC" })).toHaveAttribute("aria-selected", "true")
    expect(screen.getByText("Tradução PT-BR versão 2.0, para a versão Steam")).toBeInTheDocument()
  })

  it("selecionar_arquivo_usa_o_filtro_da_plataforma", async () => {
    falso.escolherArquivoDoJogo.mockResolvedValue(null)
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    await userEvent.click(screen.getByRole("button", { name: "Selecionar ISO" }))
    expect(falso.escolherArquivoDoJogo).toHaveBeenCalledWith(infoPs2)
  })

  it("traduzir_na_aba_pc_manda_a_plataforma", async () => {
    const traduzir = vi.fn()
    falso.escolherArquivoDoJogo.mockResolvedValue("C:/jogos/jak2")
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, aviso: null, saida_sugerida: "C:/jogos/jak2", ja_existe: false })
    render(<JogoTela jogo={jogoPs2ePc()} plataformaInicial="pc" aoVoltar={() => {}} aoTraduzir={traduzir} />)
    await userEvent.click(screen.getByRole("button", { name: "Escolher pasta do jogo" }))
    await userEvent.click(await screen.findByRole("button", { name: "Instalar tradução" }))
    expect(falso.conferirIso).toHaveBeenCalledWith("jak2-pc-ptbr", "C:/jogos/jak2")
    expect(traduzir).toHaveBeenCalledWith({ pacoteId: "jak2-pc-ptbr", plataformaId: "pc", iso: "C:/jogos/jak2", saida: "C:/jogos/jak2", sobrescrever: false })
  })
})

describe("Jogo, outra pasta", () => {
  it("escolher_outra_pasta_confere_se_o_arquivo_existe", async () => {
    const traduzir = vi.fn()
    falso.escolherArquivoDoJogo.mockResolvedValue("C:/jogos/jak2.iso")
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, aviso: null, saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", ja_existe: false })
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
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, aviso: null, saida_sugerida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso", ja_existe: false })
    falso.arquivoExiste.mockResolvedValue(true)
    render(
      <JogoTela
        jogo={jak2}
        aviso="Já existe um arquivo em D:/outra."
        pedidoAnterior={{ pacoteId: "jak2-usa-ptbr", plataformaId: "ps2", iso: "C:/jogos/jak2.iso", saida: "D:/outra\\Jak II (USA) PT-BR v1.0.iso", sobrescrever: false }}
        aoVoltar={() => {}}
        aoTraduzir={() => {}}
      />,
    )
    expect(await screen.findByText("D:/outra\\Jak II (USA) PT-BR v1.0.iso")).toBeInTheDocument()
    expect(await screen.findByRole("button", { name: "Substituir e traduzir" })).toBeInTheDocument()
    expect(screen.getByText("C:/jogos/jak2.iso")).toBeInTheDocument()
  })
})

describe("Caixa", () => {
  it("capa_do_acervo_nao_repete_a_faixa_do_ps2", () => {
    render(<Caixa nome="Jak II" capa="data:image/jpeg;base64,AAAA" plataforma={infoPs2} />)
    expect(screen.queryByText("PlayStation 2")).not.toBeInTheDocument()
  })

  it("sem_capa_desenha_a_faixa_e_o_nome", () => {
    render(<Caixa nome="Jak II" capa={null} plataforma={infoPs2} />)
    expect(screen.getByText("PlayStation 2")).toBeInTheDocument()
    expect(screen.getByText("Jak II")).toBeInTheDocument()
  })

  it("caixa_no_formato_de_cada_plataforma", () => {
    const formato = (info: typeof infoPs2) => {
      const { container, unmount } = render(<Caixa nome="Jogo" capa="data:image/jpeg;base64,AAAA" plataforma={info} />)
      const classe = (container.firstChild as HTMLElement).className
      unmount()
      return classe
    }
    expect(formato(infoPs2)).toContain("aspect-[7/10]")
    expect(formato(infoPs1)).toContain("aspect-square")
    expect(formato(infoPc)).toContain("aspect-[2/3]")
  })

  it("caixa_com_proporcao_desconhecida_usa_a_do_dvd", () => {
    const { container } = render(<Caixa nome="Jogo" capa={null} plataforma={{ ...infoPs2, proporcao_capa: "9/7" }} />)
    expect((container.firstChild as HTMLElement).className).toContain("aspect-[7/10]")
  })

  it("caixa_sem_capa_mostra_o_nome_da_plataforma", () => {
    render(<Caixa nome="Jak II" capa={null} plataforma={{ ...infoPs2, nome: "PlayStation" }} />)
    expect(screen.getByText("PlayStation")).toBeInTheDocument()
  })
})

describe("Jogo, fundo", () => {
  const comCapa = jogoPs2({ trailer_youtube: "Pj-ueiUYEbg" }, {}, "data:image/jpeg;base64,AAAA")

  it("jogo_toca_o_trailer_atras_sem_som", () => {
    render(<JogoTela jogo={comCapa} mostrarTrailers aoVoltar={() => {}} aoTraduzir={() => {}} />)
    const video = screen.getByTitle("Trailer de Jak II")
    expect(video.getAttribute("src")).toContain("youtube-nocookie.com/embed/Pj-ueiUYEbg")
    expect(video.getAttribute("src")).toContain("mute=1")
    expect(document.querySelector("[data-fundo='capa']")).toBeInTheDocument()
  })

  it("video_desligado_fica_so_a_capa_desfocada", () => {
    render(<JogoTela jogo={comCapa} mostrarTrailers={false} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.queryByTitle("Trailer de Jak II")).not.toBeInTheDocument()
    expect(document.querySelector("[data-fundo='capa']")).toBeInTheDocument()
  })

  it("sem_internet_nao_carrega_o_trailer", () => {
    render(<JogoTela jogo={comCapa} mostrarTrailers offline aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.queryByTitle("Trailer de Jak II")).not.toBeInTheDocument()
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
    render(<ProgressoTela nomeJogo="Jak II" plataforma={infoPs2} comDownload progresso={{ etapa: "baixando", fracao: 0.5 }} />)
    expect(screen.getByText("Baixando a tradução").closest("li")).toHaveAttribute("aria-current", "step")
    expect(screen.getByText("Etapa 2 de 4")).toBeInTheDocument()
  })

  it("progresso_sem_download_nao_mostra_baixando", () => {
    render(<ProgressoTela nomeJogo="Jak II" plataforma={infoPs2} progresso={{ etapa: "aplicando", fracao: 0.5 }} />)
    expect(screen.queryByText("Baixando a tradução")).not.toBeInTheDocument()
  })

  it("progresso_mostra_etapa_e_porcentagem", () => {
    render(<ProgressoTela nomeJogo="Jak II" plataforma={infoPs2} progresso={{ etapa: "aplicando", fracao: 0.42 }} />)
    expect(screen.getByText("42%")).toBeInTheDocument()
    expect(screen.getByText("Aplicando a tradução").closest("li")).toHaveAttribute("aria-current", "step")
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "42")
  })

  it("cancelar_chama_api", async () => {
    render(<ProgressoTela nomeJogo="Jak II" plataforma={infoPs2} progresso={{ etapa: "aplicando", fracao: 0.1 }} />)
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }))
    expect(falso.cancelarTraducao).toHaveBeenCalled()
  })
})

describe("Progresso, plataforma", () => {
  it("progresso_fala_do_arquivo_e_do_hash_no_ps2", () => {
    render(<ProgressoTela nomeJogo="Jak II" plataforma={infoPs2} progresso={{ etapa: "conferindo", fracao: 0 }} />)
    expect(screen.getByText("Conferindo a ISO")).toBeInTheDocument()
    expect(screen.getByText("Verificando o hash do RetroAchievements")).toBeInTheDocument()
  })

  it("progresso_sem_ra_nao_fala_de_hash", () => {
    render(<ProgressoTela nomeJogo="Jak II" plataforma={infoPc} progresso={{ etapa: "conferindo", fracao: 0 }} />)
    expect(screen.getByText("Conferindo a pasta do jogo")).toBeInTheDocument()
    expect(screen.queryByText(/RetroAchievements/)).not.toBeInTheDocument()
    expect(screen.getByText("Verificando o resultado")).toBeInTheDocument()
  })
})

describe("Pronto", () => {
  const resultado = (compativel: boolean | null): Resultado => ({
    saida: "C:/jogos/Jak II (USA) PT-BR v1.0.iso",
    hash_ra: compativel === null ? null : "f51c94dd47cc2eb385d5733389428121",
    compativel_ra: compativel,
    segundos: 12.3,
  })

  it("pronto_mostra_compatibilidade_ra", async () => {
    render(<Pronto nomeJogo="Jak II" plataforma={jak2.plataformas[0]} traducao={traducao} resultado={resultado(true)} aoVoltar={() => {}} />)
    expect(screen.getByRole("heading", { name: "ISO traduzida pronta" })).toBeInTheDocument()
    expect(screen.getByText("Compatível com RetroAchievements")).toBeInTheDocument()
    expect(screen.getByText("Fulano de Tal")).toBeInTheDocument()
    await userEvent.click(screen.getByRole("button", { name: "Abrir pasta" }))
    await waitFor(() => expect(falso.abrirPasta).toHaveBeenCalledWith("C:/jogos/Jak II (USA) PT-BR v1.0.iso"))
  })

  it("pronto_ps2_hash_mudou", () => {
    render(<Pronto nomeJogo="Jak II" plataforma={jak2.plataformas[0]} traducao={traducao} resultado={resultado(false)} aoVoltar={() => {}} />)
    expect(screen.getByText("O RetroAchievements não reconhece esta cópia")).toBeInTheDocument()
    expect(screen.queryByText(/O hash do RetroAchievements mudou/)).not.toBeInTheDocument()
  })

  it("pronto_pc", async () => {
    const r: Resultado = { saida: "C:/Steam/Jogo", hash_ra: null, compativel_ra: null, segundos: 3 }
    render(<Pronto nomeJogo="Jak II" plataforma={plataforma(infoPc, [traducao])} traducao={traducao} resultado={r} aoVoltar={() => {}} />)
    expect(screen.getByRole("heading", { name: "Tradução instalada na pasta do jogo." })).toBeInTheDocument()
    expect(screen.queryByText(/RetroAchievements/)).not.toBeInTheDocument()
    expect(screen.queryByRole("button", { name: "Abrir pasta" })).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole("button", { name: "Abrir a pasta do jogo" }))
    await waitFor(() => expect(falso.abrirPasta).toHaveBeenCalledWith("C:/Steam/Jogo"))
  })
})

describe("Jogo, selos", () => {
  it("selo_ra_compativel", () => {
    render(<JogoTela jogo={comTraducao({ ra: "compativel" })} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.getByText("Compatível com RetroAchievements")).toBeInTheDocument()
  })

  it("selo_ra_nao_reconhece", () => {
    render(<JogoTela jogo={comTraducao({ ra: "nao_reconhece" })} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.getByText("O RetroAchievements não reconhece esta cópia")).toBeInTheDocument()
    expect(screen.queryByText("Compatível com RetroAchievements")).not.toBeInTheDocument()
  })

  it("selo_contem_programa", () => {
    const jogo: Jogo = { id: "j", nome: "Jogo", plataformas: [plataforma(infoPc, [{ ...traducaoJak2, ra: null, contem_programa: true }])] }
    render(<JogoTela jogo={jogo} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    const selo = screen.getByText("Contém programa")
    expect(selo).toHaveAttribute("title", "Esta tradução instala um programa (por exemplo, um carregador de mods) na pasta do jogo.")
    expect(screen.queryByText(/RetroAchievements/)).not.toBeInTheDocument()
  })

  it("sem_programa_nao_tem_selo", () => {
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.queryByText("Contém programa")).not.toBeInTheDocument()
  })
})

describe("Jogo, PC", () => {
  const pc: Jogo = { id: "j", nome: "Jogo", plataformas: [plataforma(infoPc, [{ ...traducaoJak2, id: "j-pc", ra: null }])] }

  it("pc_pede_pasta", async () => {
    falso.escolherArquivoDoJogo.mockResolvedValue("C:/Steam/Jogo")
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, aviso: null, saida_sugerida: "C:/Steam/Jogo", ja_existe: false })
    render(<JogoTela jogo={pc} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    await userEvent.click(screen.getByRole("button", { name: "Escolher pasta do jogo" }))
    expect(falso.escolherArquivoDoJogo).toHaveBeenCalledWith(infoPc)
    expect(await screen.findByRole("button", { name: "Instalar tradução" })).toBeEnabled()
    expect(screen.queryByText(/será gravada em/)).not.toBeInTheDocument()
    expect(screen.queryByRole("button", { name: "Escolher outra pasta" })).not.toBeInTheDocument()
  })

  it("pc_mostra_aviso_de_versao", async () => {
    const aviso = "Esta pasta tem uma versão do jogo diferente da testada (1.0.3). A tradução pode não funcionar direito."
    falso.escolherArquivoDoJogo.mockResolvedValue("C:/Steam/Jogo")
    falso.conferirIso.mockResolvedValue({ serve: true, mensagem: null, aviso, saida_sugerida: "C:/Steam/Jogo", ja_existe: false })
    render(<JogoTela jogo={pc} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    await userEvent.click(screen.getByRole("button", { name: "Escolher pasta do jogo" }))
    expect(await screen.findByText(aviso)).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Instalar tradução" })).toBeEnabled()
  })

  it("remover_traducao", async () => {
    falso.escolherArquivoDoJogo.mockResolvedValue("C:/Steam/Jogo")
    falso.removerTraducao.mockResolvedValue({ removidos: 1, restaurados: 1, mantidos: [] })
    render(<JogoTela jogo={pc} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    await userEvent.click(screen.getByRole("button", { name: "Remover tradução" }))
    expect(falso.escolherArquivoDoJogo).toHaveBeenCalledWith(infoPc)
    expect(falso.removerTraducao).toHaveBeenCalledWith("j-pc", "C:/Steam/Jogo")
    expect(await screen.findByText("Tradução removida.")).toBeInTheDocument()
    expect(screen.queryByText(/atualizados pela loja/)).not.toBeInTheDocument()
  })

  it("remover_traducao_com_arquivo_da_loja", async () => {
    falso.escolherArquivoDoJogo.mockResolvedValue("C:/Steam/Jogo")
    falso.removerTraducao.mockResolvedValue({ removidos: 0, restaurados: 0, mantidos: ["dados/menu.txt"] })
    render(<JogoTela jogo={pc} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    await userEvent.click(screen.getByRole("button", { name: "Remover tradução" }))
    expect(await screen.findByText("Tradução removida.")).toBeInTheDocument()
    expect(screen.getByText("Alguns arquivos foram atualizados pela loja e ficaram como estão.")).toBeInTheDocument()
  })

  it("remover_traducao_com_erro", async () => {
    falso.escolherArquivoDoJogo.mockResolvedValue("C:/Steam/Jogo")
    falso.removerTraducao.mockRejectedValue("Esta tradução não está instalada nesta pasta.")
    render(<JogoTela jogo={pc} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    await userEvent.click(screen.getByRole("button", { name: "Remover tradução" }))
    expect(await screen.findByText("Esta tradução não está instalada nesta pasta.")).toBeInTheDocument()
    expect(screen.queryByText("Tradução removida.")).not.toBeInTheDocument()
  })

  it("ps2_nao_tem_remover", () => {
    render(<JogoTela jogo={jak2} aoVoltar={() => {}} aoTraduzir={() => {}} />)
    expect(screen.queryByRole("button", { name: "Remover tradução" })).not.toBeInTheDocument()
  })
})
