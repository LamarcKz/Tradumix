import { fileURLToPath } from "node:url"

import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig, type Plugin } from "vite"

// A prévia: as mesmas telas do app num site estático (dist-previa/), lendo o lista.json que o
// `tradumix-cli previa` gera. O módulo ./api (a ponte com o Rust) é trocado por src/api-previa.ts.
const apiPrevia = fileURLToPath(new URL("./src/api-previa.ts", import.meta.url))

// Aberta direto do disco (file://), o navegador não carrega o app: fica este aviso no lugar da tela branca.
const avisoSemServidor: Plugin = {
  name: "aviso-sem-servidor",
  transformIndexHtml: (html) =>
    html.replace(
      '<div id="raiz"></div>',
      '<div id="raiz"><p style="color:#e5e5e5;font-family:sans-serif;padding:40px;max-width:60ch">' +
        "A prévia não abriu. Ela precisa ser aberta pelo teste da oficina, que a serve num endereço local; " +
        "aberta direto do arquivo, o navegador bloqueia o app.</p></div>",
    ),
}

export default defineConfig({
  mode: "previa",
  base: "./",
  plugins: [react(), tailwindcss(), avisoSemServidor],
  resolve: {
    alias: [{ find: /^\.{1,2}\/api$/, replacement: apiPrevia }],
  },
  build: {
    outDir: "dist-previa",
    emptyOutDir: true,
  },
})
