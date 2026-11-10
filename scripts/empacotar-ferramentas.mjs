// Empacota as ferramentas de teste da oficina: o tradumix-cli e a prévia das telas.
//
//   node scripts/empacotar-ferramentas.mjs <versao> <pasta-saida>
//
// Gera na pasta de saída:
// - tradumix-cli-<versao>-windows-x64.zip: tradumix-cli.exe e a licença (LICENSE);
// - tradumix-previa-<versao>.zip: o dist-previa/ (npm run build:previa), sem lista.json;
// - ferramentas-<versao>.json: { versao, arquivos: [{ nome, sha256, tamanho }] }.
// Usa o tradumix-cli.exe de target/release (cargo build --release -p tradumix-cli).
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { pathToFileURL } from "node:url"

const sha256 = (arquivo) => createHash("sha256").update(readFileSync(arquivo)).digest("hex")

/** Zipa `itens` de dentro de `pasta` com o tar do Windows. */
function zipar(zip, pasta, itens) {
  const tar = join(process.env.SystemRoot ?? "C:\Windows", "System32", "tar.exe")
  const r = spawnSync(tar, ["-a", "-cf", zip, "-C", pasta, ...itens], { stdio: "inherit" })
  if (r.status !== 0) throw new Error(`o tar falhou ao gerar ${zip}`)
}

/** Gera os dois .zip e o .json na pasta `saida`; devolve os nomes, na ordem. */
export function empacotarFerramentas({ versao, saida, exe, previa, licenca }) {
  if (!existsSync(exe)) throw new Error(`Falta ${exe}: rode cargo build --release -p tradumix-cli.`)
  if (!existsSync(join(previa, "index.html"))) throw new Error(`Falta a prévia em ${previa}: rode npm run build:previa.`)
  mkdirSync(saida, { recursive: true })
  const nomeCli = `tradumix-cli-${versao}-windows-x64.zip`
  const nomePrevia = `tradumix-previa-${versao}.zip`
  const nomeJson = `ferramentas-${versao}.json`

  const temp = mkdtempSync(join(tmpdir(), "tradumix-cli-"))
  try {
    copyFileSync(exe, join(temp, "tradumix-cli.exe"))
    copyFileSync(licenca, join(temp, "LICENSE"))
    zipar(join(saida, nomeCli), temp, ["tradumix-cli.exe", "LICENSE"])
  } finally {
    rmSync(temp, { recursive: true, force: true })
  }
  // O lista.json de um teste local não vai: a oficina gera o dela.
  const itensPrevia = readdirSync(previa).filter((n) => n !== "lista.json")
  zipar(join(saida, nomePrevia), previa, itensPrevia)

  const arquivos = [nomeCli, nomePrevia].map((nome) => {
    const arquivo = join(saida, nome)
    return { nome, sha256: sha256(arquivo), tamanho: statSync(arquivo).size }
  })
  writeFileSync(join(saida, nomeJson), JSON.stringify({ versao, arquivos }, null, 2) + "\n")
  return [nomeCli, nomePrevia, nomeJson]
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [versao, saida] = process.argv.slice(2)
  if (!versao || !saida) {
    console.error("uso: node scripts/empacotar-ferramentas.mjs <versao> <pasta-saida>")
    process.exit(2)
  }
  const nomes = empacotarFerramentas({
    versao,
    saida,
    exe: join("target", "release", "tradumix-cli.exe"),
    previa: "dist-previa",
    licenca: "LICENSE",
  })
  for (const nome of nomes) console.log(`${nome}  ${statSync(join(saida, nome)).size} bytes`)
}
