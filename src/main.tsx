import { StrictMode } from "react"
import { createRoot } from "react-dom/client"

createRoot(document.getElementById("raiz")!).render(
  <StrictMode>
    <main />
  </StrictMode>,
)
