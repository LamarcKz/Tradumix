// Gera em dist-release/ o que vai numa release do Tradumix (adaptado do PlayTrove). Rode com `npm run dist`.
//
// - Tradumix-Setup-X.Y.Z.exe: o instalador (só para o usuário, sem pedir administrador).
// - Tradumix-X.Y.Z-portable.zip: Tradumix.exe, xdelta3.exe, portable.txt e a licença do xdelta3 (os dados
//   ficam na pasta "dados", ali do lado).
// - latest.json: o que o app lê para avisar da versão nova, com a assinatura do instalador.
//
// A assinatura usa a chave do Tradumix, que fica fora do repositório: a variável TAURI_SIGNING_PRIVATE_KEY
// ou, sem ela, o arquivo .tauri/tradumix.key na pasta do usuário.
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs"
import { homedir } from "node:os"
import { join } from "node:path"

import { ambienteRelease } from "./ambiente-release.mjs"

const REPOSITORIO = "https://github.com/LamarcKz/Tradumix"
const { version } = JSON.parse(readFileSync("package.json", "utf8"))
const release = join("target", "release")
const instaladorGerado = join(release, "bundle", "nsis", `Tradumix_${version}_x64-setup.exe`)
const nomeInstalador = `Tradumix-Setup-${version}.exe`
const nomePortatil = `Tradumix-${version}-portable.zip`
const saida = "dist-release"

/** Roda um programa e para tudo se ele falhar. O `npx` só roda pelo terminal (é um .cmd no Windows). */
function rodar(comando, args, env = {}) {
  const pelaShell = comando === "npx"
  const r = pelaShell
    ? spawnSync([comando, ...args].join(" "), { stdio: "inherit", shell: true, env: { ...process.env, ...env } })
    : spawnSync(comando, args, { stdio: "inherit", env: { ...process.env, ...env } })
  if (r.status !== 0) process.exit(r.status ?? 1)
}

const sha256 = (arquivo) => createHash("sha256").update(readFileSync(arquivo)).digest("hex")

// 1. O xdelta3 conferido, o build de release e o instalador assinado (o .sig ao lado dele).
const chave = process.env.TAURI_SIGNING_PRIVATE_KEY ?? join(homedir(), ".tauri", "tradumix.key")
if (!process.env.TAURI_SIGNING_PRIVATE_KEY && !existsSync(chave)) {
  console.error("Falta a chave que assina as versões (.tauri/tradumix.key na pasta do usuário).")
  process.exit(1)
}
rodar("node", ["scripts/baixar-xdelta3.mjs"])
rodar("npx", ["tauri", "build"], {
  ...ambienteRelease(process.env, homedir(), process.cwd()),
  TAURI_SIGNING_PRIVATE_KEY: chave,
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD ?? "",
})

// 2. O programa não pode levar o caminho da pasta do usuário (o remap acima tira; aqui se confere).
const exe = join(release, "Tradumix.exe")
const pastaDoUsuario = homedir()
const bytes = readFileSync(exe)
for (const forma of new Set([pastaDoUsuario, pastaDoUsuario.replaceAll("\\", "/")])) {
  if (bytes.includes(Buffer.from(forma, "utf8")) || bytes.includes(Buffer.from(forma, "utf16le"))) {
    console.error("O Tradumix.exe ainda tem o caminho da pasta do usuário. Nada foi gerado.")
    process.exit(1)
  }
}

rmSync(saida, { recursive: true, force: true })
mkdirSync(saida, { recursive: true })
const instalador = join(saida, nomeInstalador)
copyFileSync(instaladorGerado, instalador)
const assinatura = readFileSync(`${instaladorGerado}.sig`, "utf8").trim()

// 3. A versão portátil, num .zip feito pelo tar do Windows.
const pastaPortatil = join(saida, "portatil")
mkdirSync(join(pastaPortatil, "licencas"), { recursive: true })
copyFileSync(exe, join(pastaPortatil, "Tradumix.exe"))
copyFileSync(join(release, "xdelta3.exe"), join(pastaPortatil, "xdelta3.exe"))
copyFileSync(join("build", "portable.txt"), join(pastaPortatil, "portable.txt"))
copyFileSync(join("src-tauri", "licencas", "xdelta3-LICENSE.txt"), join(pastaPortatil, "licencas", "xdelta3-LICENSE.txt"))
const portatil = join(saida, nomePortatil)
const tar = join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe")
rodar(tar, ["-a", "-cf", portatil, "-C", pastaPortatil, "Tradumix.exe", "xdelta3.exe", "portable.txt", "licencas"])
rmSync(pastaPortatil, { recursive: true, force: true })

// 4. O latest.json, que o app lê na última release.
const latest = {
  version,
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": { signature: assinatura, url: `${REPOSITORIO}/releases/download/v${version}/${nomeInstalador}` },
  },
}
writeFileSync(join(saida, "latest.json"), JSON.stringify(latest, null, 2) + "\n")

console.log(`\nPronto em ${saida}/:`)
for (const nome of [nomeInstalador, nomePortatil, "latest.json"]) {
  const arquivo = join(saida, nome)
  console.log(`  ${nome}  ${statSync(arquivo).size} bytes  SHA-256 ${sha256(arquivo)}`)
}
