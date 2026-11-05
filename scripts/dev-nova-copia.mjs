// O comando `npm run dev:nova-copia`: joga fora a cópia dos dados que o npm run dev usa
// (%APPDATA%\io.github.tradumix Dev). Na próxima vez que o npm run dev abrir, ele copia de novo os
// dados do app instalado (%APPDATA%\io.github.tradumix), que este comando nunca toca.
import { existsSync, rmSync } from "node:fs"
import { join } from "node:path"

const appData = process.env.APPDATA
if (!appData) {
  console.error("Não achei a pasta %APPDATA% do Windows.")
  process.exit(1)
}

const pastaDev = join(appData, "io.github.tradumix Dev")
if (!existsSync(pastaDev)) {
  console.log("Ainda não há cópia: o npm run dev faz uma quando abrir.")
  process.exit(0)
}

try {
  rmSync(pastaDev, { recursive: true, force: true })
} catch {
  console.error("Não deu para apagar a cópia: feche o npm run dev e rode de novo.")
  process.exit(1)
}
console.log("Pronto: na próxima vez que o npm run dev abrir, ele copia de novo os dados do app instalado.")
