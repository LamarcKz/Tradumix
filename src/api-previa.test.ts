import { afterEach, expect, it, vi } from "vitest"

import * as previa from "./api-previa"
import { jogoPs2, listaDe, infoPs2 } from "./testes/dados"

const SEM_LISTA = "A prévia não achou o lista.json. Rode o teste da oficina de novo."
const SEM_INSTALAR = "Na prévia não dá para instalar. O teste da oficina já instalou pelo mesmo código do app."

afterEach(() => vi.unstubAllGlobals())

it("api-previa lista", async () => {
  const lista = listaDe([jogoPs2()])
  const buscar = vi.fn(async () => new Response(JSON.stringify(lista)))
  vi.stubGlobal("fetch", buscar)
  expect(await previa.listarJogos()).toEqual(lista)
  expect(await previa.sincronizarCatalogo()).toEqual(lista)
  expect(buscar).toHaveBeenCalledWith("./lista.json", expect.anything())
})

it("api-previa sem lista", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => Promise.reject(new TypeError("Failed to fetch"))))
  await expect(previa.listarJogos()).rejects.toBe(SEM_LISTA)
})

it("api-previa lista 404", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("não achei", { status: 404 })))
  await expect(previa.listarJogos()).rejects.toBe(SEM_LISTA)
})

it("api-previa lista quebrada", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("{ quebrado")))
  await expect(previa.sincronizarCatalogo()).rejects.toBe(SEM_LISTA)
})

it("api-previa traduzir", async () => {
  await expect(previa.traduzir("x", "a", "b", false)).rejects.toBe(SEM_INSTALAR)
  await expect(previa.conferirIso("x", "a")).rejects.toBe(SEM_INSTALAR)
  await expect(previa.removerTraducao("x", "a")).rejects.toBe(SEM_INSTALAR)
  expect(await previa.escolherArquivoDoJogo(infoPs2)).toBeNull()
  expect(await previa.escolherPasta()).toBeNull()
})

it("api-previa sem atualizacao nem janela", async () => {
  expect(await previa.procurarVersaoNova()).toBeNull()
  expect(await previa.versaoDoApp()).toEqual({ versao: "prévia", portatil: false })
  await previa.minimizarJanela()
  await previa.fecharJanela()
  const parar = previa.aoProgresso(() => {})
  parar()
  expect(await previa.arquivoExiste("x")).toBe(false)
})
