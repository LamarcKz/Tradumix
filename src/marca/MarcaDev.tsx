/**
 * Marca DEV embaixo do logo: aparece no `npm run dev` (e nos testes) e nunca no app instalado,
 * para não confundir a versão em desenvolvimento com a que está funcionando de verdade.
 * O Vite põe `import.meta.env.DEV = false` no build.
 */
export function MarcaDev({ ativa = import.meta.env.DEV }: { ativa?: boolean }) {
  if (!ativa) return null
  return (
    <span
      title="Versão de desenvolvimento (npm run dev), não o app instalado"
      className="pointer-events-none absolute -bottom-2 left-1/2 -translate-x-1/2 rounded bg-dev px-1 text-[9px] font-bold leading-[14px] text-black"
    >
      DEV
    </span>
  )
}
