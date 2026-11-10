import { useEffect, useState } from "react"

import * as api from "../api"
import type { VersaoDoApp } from "../tipos"

const link = "underline decoration-sutil underline-offset-4 hover:decoration-texto"

/** Versão do app, de onde vem cada parte e as licenças. */
export function Sobre({ aoVoltar }: { aoVoltar: () => void }) {
  const [app, setApp] = useState<VersaoDoApp | null>(null)

  useEffect(() => {
    api.versaoDoApp().then(setApp).catch(() => {})
  }, [])

  return (
    <section className="max-w-2xl px-10 pb-16 pt-6">
      <button onClick={aoVoltar} className="text-sm text-suave hover:text-texto">
        ← Configurações
      </button>
      <h1 className="titulo mt-8 text-5xl">Sobre</h1>
      {app && (
        <p className="mt-3 text-suave">
          {app.portatil ? `Versão ${app.versao} (portátil)` : `Versão ${app.versao}`}
        </p>
      )}
      <p className="mt-6 leading-relaxed text-suave">
        O Tradumix aplica traduções de jogos na sua própria cópia, no seu PC. Nos jogos com RetroAchievements, a tradução não
        muda o hash, e as conquistas continuam funcionando.
        Cada tradução tem os créditos de quem a fez na página do jogo.
      </p>

      <h2 className="mt-10 border-t border-borda pt-6 font-semibold">Licenças e créditos</h2>
      <ul className="mt-4 space-y-4 text-sm">
        <li>
          <p>Tradumix — MIT</p>
          <a href="https://github.com/LamarcKz/Tradumix" target="_blank" rel="noreferrer" className={link}>
            Código do Tradumix
          </a>
          {" · "}
          <a href="https://github.com/LamarcKz/Tradumix-catalogo" target="_blank" rel="noreferrer" className={link}>
            Catálogo de traduções
          </a>
        </li>
        <li>
          <p>xdelta3 3.2.2 — Apache 2.0, Joshua MacDonald</p>
          <p className="text-suave">Aplica o patch na imagem do jogo (ISO ou .bin). Vai junto com o app, compilado do código oficial; a licença está em licencas/xdelta3-LICENSE.txt, na pasta do Tradumix.</p>
          <a href="https://github.com/jmacd/xdelta" target="_blank" rel="noreferrer" className={link}>
            Código do xdelta3
          </a>
        </li>
        <li>
          <p>Capas: acervo ps2-covers, usado pelo PCSX2</p>
          <a href="https://github.com/xlenore/ps2-covers" target="_blank" rel="noreferrer" className={link}>
            Acervo ps2-covers
          </a>
        </li>
        <li>
          <p>Trailers: player do YouTube</p>
          <p className="text-suave">Os vídeos tocam no player oficial (youtube-nocookie.com); dá para desligar em Configurações.</p>
        </li>
      </ul>
    </section>
  )
}
