import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"

import { afterEach, expect, it } from "vitest"

import { empacotarFerramentas } from "./empacotar-ferramentas.mjs"

const pastas: string[] = []
afterEach(() => {
  for (const p of pastas.splice(0)) rmSync(p, { recursive: true, force: true })
})

const sha256 = (arquivo: string) => createHash("sha256").update(readFileSync(arquivo)).digest("hex")

/** Lista o conteúdo de um .zip com o tar do Windows. */
function conteudo(zip: string): string[] {
  const tar = join(process.env.SystemRoot ?? "C:\Windows", "System32", "tar.exe")
  const r = spawnSync(tar, ["-tf", zip], { encoding: "utf8" })
  expect(r.status).toBe(0)
  return r.stdout.split(/\r?\n/).filter(Boolean).map((l) => l.replace(/\/$/, "")).sort()
}

it("empacotar gera zip e json com sha256 certos", () => {
  const raiz = mkdtempSync(join(tmpdir(), "ferramentas-"))
  pastas.push(raiz)
  const exe = join(raiz, "tradumix-cli.exe")
  writeFileSync(exe, "MZ falso")
  const licenca = join(raiz, "LICENSE")
  writeFileSync(licenca, "MIT")
  const previa = join(raiz, "dist-previa")
  mkdirSync(join(previa, "assets"), { recursive: true })
  writeFileSync(join(previa, "index.html"), "<html></html>")
  writeFileSync(join(previa, "assets", "a.js"), "console.log(1)")
  writeFileSync(join(previa, "lista.json"), "{}") // sobra de um teste local: não vai no pacote
  const saida = join(raiz, "saida")

  const nomes = empacotarFerramentas({ versao: "dev-abc1234", saida, exe, previa, licenca })

  expect(nomes).toEqual(["tradumix-cli-dev-abc1234-windows-x64.zip", "tradumix-previa-dev-abc1234.zip", "ferramentas-dev-abc1234.json"])
  expect(conteudo(join(saida, "tradumix-cli-dev-abc1234-windows-x64.zip"))).toEqual(["LICENSE", "tradumix-cli.exe"])
  expect(conteudo(join(saida, "tradumix-previa-dev-abc1234.zip"))).toEqual(["assets", "assets/a.js", "index.html"])

  const json = JSON.parse(readFileSync(join(saida, "ferramentas-dev-abc1234.json"), "utf8"))
  expect(json.versao).toBe("dev-abc1234")
  expect(json.arquivos.map((a: { nome: string }) => a.nome)).toEqual(["tradumix-cli-dev-abc1234-windows-x64.zip", "tradumix-previa-dev-abc1234.zip"])
  for (const a of json.arquivos) {
    const arquivo = join(saida, a.nome)
    expect(a.sha256).toBe(sha256(arquivo))
    expect(a.tamanho).toBe(readFileSync(arquivo).length)
  }
})

it("empacotar sem a previa avisa", () => {
  const raiz = mkdtempSync(join(tmpdir(), "ferramentas-"))
  pastas.push(raiz)
  writeFileSync(join(raiz, "x.exe"), "MZ")
  writeFileSync(join(raiz, "LICENSE"), "MIT")
  expect(() =>
    empacotarFerramentas({ versao: "1.0.0", saida: join(raiz, "s"), exe: join(raiz, "x.exe"), previa: join(raiz, "nada"), licenca: join(raiz, "LICENSE") }),
  ).toThrow(/npm run build:previa/)
})
