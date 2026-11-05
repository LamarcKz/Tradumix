import { render, screen } from "@testing-library/react"
import { expect, it } from "vitest"

import { MarcaDev } from "./MarcaDev"

it("marca_dev_aparece_no_npm_run_dev", () => {
  render(<MarcaDev ativa />)
  expect(screen.getByText("DEV")).toHaveAttribute("title", "Versão de desenvolvimento (npm run dev), não o app instalado")
})

it("marca_dev_some_no_app_instalado", () => {
  const { container } = render(<MarcaDev ativa={false} />)
  expect(container).toBeEmptyDOMElement()
})
