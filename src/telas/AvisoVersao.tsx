import { useEffect, useState } from "react"

import * as api from "../api"
import type { VersaoNova } from "../tipos"

/**
 * Faixa discreta embaixo do topo quando sai uma versão nova do Tradumix. Procura uma vez ao abrir; sem
 * versão nova, sem internet ou com qualquer erro, não mostra nada. A pessoa decide se atualiza; fechar
 * esconde o aviso até a próxima vez que o app abrir. Na versão portátil, o botão abre a página de
 * download (o instalador não troca a portátil).
 */
export function AvisoVersao() {
  const [nova, setNova] = useState<VersaoNova | null>(null)
  const [fechado, setFechado] = useState(false)
  const [novidades, setNovidades] = useState(false)
  const [atualizando, setAtualizando] = useState(false)
  const [erro, setErro] = useState<string | null>(null)

  useEffect(() => {
    let ativo = true
    api
      .procurarVersaoNova()
      .then((v) => ativo && setNova(v))
      .catch(() => {})
    return () => {
      ativo = false
    }
  }, [])

  if (!nova || fechado) return null

  async function atualizar() {
    if (!nova) return
    setErro(null)
    setAtualizando(!nova.portatil)
    try {
      await api.instalarVersaoNova()
    } catch (e) {
      setAtualizando(false)
      setErro(`Não deu para atualizar: ${String(e)}`)
    }
  }

  return (
    <section aria-label="Versão nova" className="mx-10 mt-4 rounded-md border border-borda bg-superficie px-4 py-3 text-sm">
      <div className="flex flex-wrap items-center gap-3">
        <p className="font-semibold">Versão {nova.versao} disponível</p>
        {atualizando && <p className="text-suave">Baixando a versão {nova.versao}… o Tradumix reinicia sozinho.</p>}
        <div className="flex-1" />
        {nova.notas && (
          <button onClick={() => setNovidades(!novidades)} className="text-suave hover:text-texto">
            {novidades ? "Esconder novidades" : "Ver novidades"}
          </button>
        )}
        <button
          onClick={atualizar}
          disabled={atualizando}
          className="rounded-md bg-destaque px-3 py-1.5 font-semibold text-fundo hover:bg-[#d5d5d5] disabled:opacity-60"
        >
          {nova.portatil ? "Baixar" : "Atualizar"}
        </button>
        <button onClick={() => setFechado(true)} aria-label="Fechar aviso" className="px-1 text-lg leading-none text-sutil hover:text-texto">
          ×
        </button>
      </div>
      {novidades && <p className="mt-2 whitespace-pre-line text-suave">{nova.notas}</p>}
      {erro && (
        <p role="alert" className="mt-2 text-erro">
          {erro}
        </p>
      )}
    </section>
  )
}
