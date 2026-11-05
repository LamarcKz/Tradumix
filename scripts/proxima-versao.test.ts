import { expect, it } from "vitest"

import { proximaVersao } from "./proxima-versao.mjs"

it("feat_sobe_o_do_meio", () => {
  const r = proximaVersao("v0.1.0", ["feat(app): tela Sobre", "fix: corrige o filtro", "docs: README"])
  expect(r.proxima).toBe("0.2.0")
  expect(r.tipo).toBe("novidades")
  expect(r.novidades).toEqual(["Tela Sobre"])
  expect(r.correcoes).toEqual(["Corrige o filtro"])
})

it("so_fix_sobe_o_ultimo", () => {
  expect(proximaVersao("v0.2.0", ["fix(app): corrige o botão", "chore: organiza"]).proxima).toBe("0.2.1")
})

it("so_docs_nao_pede_versao", () => {
  expect(proximaVersao("v0.2.1", ["docs: atualiza o guia", "refactor: separa funções", "ci: testes"]).proxima).toBeNull()
})

it("mudanca_que_quebra_conta_como_novidade", () => {
  expect(proximaVersao("v0.2.1", ["refactor!: muda o formato da ficha"]).proxima).toBe("0.3.0")
})
