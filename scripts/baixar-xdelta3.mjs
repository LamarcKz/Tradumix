// Baixa o xdelta3 3.2.2 (Windows x86-64) que vai dentro do Tradumix, confere o SHA-256 e grava como
// sidecar do Tauri. Ele é compilado do código oficial (github.com/jmacd/xdelta) pelo workflow
// .github/workflows/xdelta3.yml, que publica a release xdelta3-v3.2.2 deste repositório.
// O SHA-256 abaixo é o daquela release: se o arquivo mudar, nada é gravado.
// Uso: npm run xdelta3
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const URL_EXE = "https://github.com/LamarcKz/Tradumix/releases/download/xdelta3-v3.2.2/xdelta3-3.2.2-windows-x86_64.exe";
const SHA_EXE = "df624a216a7f0f96dc7ac052b462f28657ef0a6e9c78d3008bd687cbac9a0e63";

const raiz = join(dirname(fileURLToPath(import.meta.url)), "..");
const destino = join(raiz, "src-tauri", "binaries", "xdelta3-x86_64-pc-windows-msvc.exe");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

if (existsSync(destino) && sha256(readFileSync(destino)) === SHA_EXE) {
  console.log("xdelta3 já está em src-tauri/binaries e confere com o SHA-256 esperado.");
  process.exit(0);
}

console.log(`Baixando ${URL_EXE}`);
const resposta = await fetch(URL_EXE);
if (!resposta.ok) {
  console.error(`Falha no download: HTTP ${resposta.status}`);
  process.exit(1);
}
const exe = Buffer.from(await resposta.arrayBuffer());
if (sha256(exe) !== SHA_EXE) {
  console.error("O SHA-256 do xdelta3.exe não confere. Nada foi gravado.");
  process.exit(1);
}
mkdirSync(dirname(destino), { recursive: true });
writeFileSync(destino, exe);
console.log("xdelta3 gravado em src-tauri/binaries/xdelta3-x86_64-pc-windows-msvc.exe");
