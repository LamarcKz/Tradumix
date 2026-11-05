// Baixa o xdelta3 oficial (3.2.2, Windows x86-64), confere o SHA-256 e grava como sidecar do Tauri.
// Uso: npm run xdelta3
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync, copyFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const URL_ZIP = "https://github.com/jmacd/xdelta/releases/download/v3.2.2/xdelta3-3.2.2-windows-x86_64.zip";
const SHA_ZIP = "d48f7651d86b78fdca397057b46601f89131f37219675df736db70af44ec3379";
const SHA_EXE = "732980d52f9220dc81817ccdd0c295eadce8a0048a24c2a771a73f791ac824e9";

const raiz = join(dirname(fileURLToPath(import.meta.url)), "..");
const destino = join(raiz, "src-tauri", "binaries", "xdelta3-x86_64-pc-windows-msvc.exe");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

if (existsSync(destino) && sha256(readFileSync(destino)) === SHA_EXE) {
  console.log("xdelta3 já está em src-tauri/binaries e confere com o SHA-256 esperado.");
  process.exit(0);
}

console.log(`Baixando ${URL_ZIP}`);
const resposta = await fetch(URL_ZIP);
if (!resposta.ok) {
  console.error(`Falha no download: HTTP ${resposta.status}`);
  process.exit(1);
}
const zip = Buffer.from(await resposta.arrayBuffer());
if (sha256(zip) !== SHA_ZIP) {
  console.error("O SHA-256 do zip não confere. Nada foi gravado.");
  process.exit(1);
}

const temp = mkdtempSync(join(tmpdir(), "tradumix-xdelta3-"));
try {
  const arquivoZip = join(temp, "xdelta3.zip");
  writeFileSync(arquivoZip, zip);
  // O tar do Windows (bsdtar) abre zip; o GNU tar do Git Bash confundiria "C:" com uma máquina.
  const tar = join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe");
  execFileSync(tar, ["-xf", arquivoZip, "-C", temp]);
  const exe = join(temp, "xdelta3-3.2.2-windows-x86_64", "xdelta3.exe");
  if (sha256(readFileSync(exe)) !== SHA_EXE) {
    console.error("O SHA-256 do xdelta3.exe não confere. Nada foi gravado.");
    process.exit(1);
  }
  mkdirSync(dirname(destino), { recursive: true });
  copyFileSync(exe, destino);
  console.log("xdelta3 gravado em src-tauri/binaries/xdelta3-x86_64-pc-windows-msvc.exe");
} finally {
  rmSync(temp, { recursive: true, force: true });
}
