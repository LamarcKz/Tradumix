// A ponte das telas na prévia: no lugar do app em Rust, lê o lista.json que o `tradumix-cli previa`
// gerou ao lado do index.html. Nada é instalado: a prévia só mostra como a tradução vai aparecer.
// O build da prévia (vite.previa.config.ts) troca o módulo ./api por este.
import type { InfoPlataforma, ListaJogos, Progresso, Remocao, Resultado, VersaoDoApp, VersaoNova, Conferencia } from "./tipos"

const SEM_LISTA = "A prévia não achou o lista.json. Rode o teste da oficina de novo."
const SEM_INSTALAR = "Na prévia não dá para instalar. O teste da oficina já instalou pelo mesmo código do app."

async function lerLista(): Promise<ListaJogos> {
  try {
    const resposta = await fetch("./lista.json", { cache: "no-store" })
    if (!resposta.ok) throw new Error(String(resposta.status))
    return (await resposta.json()) as ListaJogos
  } catch {
    // Como o app em Rust, a recusa é o texto da mensagem (as telas mostram String(erro)).
    return Promise.reject(SEM_LISTA)
  }
}

const recusar = <T,>(): Promise<T> => Promise.reject(SEM_INSTALAR)
const nada = async () => {}

const api = {
  listarJogos: lerLista,
  sincronizarCatalogo: lerLista,
  definirMostrarTrailers: (_mostrar: boolean) => lerLista(),
  conferirIso: (_pacoteId: string, _iso: string) => recusar<Conferencia>(),
  traduzir: (_pacoteId: string, _iso: string, _saida: string, _sobrescrever: boolean) => recusar<Resultado>(),
  removerTraducao: (_pacoteId: string, _pasta: string) => recusar<Remocao>(),
  cancelarTraducao: nada,
  arquivoExiste: async (_caminho: string) => false,
  abrirPasta: async (_caminho: string) => {},
  versaoDoApp: async (): Promise<VersaoDoApp> => ({ versao: "prévia", portatil: false }),
  procurarVersaoNova: async (): Promise<VersaoNova | null> => null,
  instalarVersaoNova: nada,
  minimizarJanela: nada,
  alternarMaximizarJanela: nada,
  fecharJanela: nada,
  aoProgresso: (_cb: (p: Progresso) => void) => () => {},
  escolherArquivoDoJogo: async (_info: InfoPlataforma): Promise<string | null> => null,
  escolherPasta: async (): Promise<string | null> => null,
} satisfies typeof import("./api")

export const {
  listarJogos,
  sincronizarCatalogo,
  definirMostrarTrailers,
  conferirIso,
  traduzir,
  removerTraducao,
  cancelarTraducao,
  arquivoExiste,
  abrirPasta,
  versaoDoApp,
  procurarVersaoNova,
  instalarVersaoNova,
  minimizarJanela,
  alternarMaximizarJanela,
  fecharJanela,
  aoProgresso,
  escolherArquivoDoJogo,
  escolherPasta,
} = api
