import react from "@vitejs/plugin-react"
import { defineConfig } from "vitest/config"

// Testes das telas, sem abrir o app: src/**/*.test.tsx. O módulo src/api.ts é trocado por um falso.
export default defineConfig({
  plugins: [react()],
  test: {
    include: ["src/**/*.test.tsx", "src/**/*.test.ts"],
    environment: "jsdom",
    globals: true,
    setupFiles: ["src/testes/preparar.ts"],
  },
})
