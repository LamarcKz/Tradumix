import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

// A interface em React: servida pelo Vite no "npm run dev" (porta fixa 1430, a do tauri.conf.json)
// e gerada em dist/ no build, que vai dentro do app.
export default defineConfig({
  base: "./",
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1430,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**", "**/target/**"] },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
})
