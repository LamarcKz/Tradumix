export function proximaVersao(
  ultimaTag: string,
  assuntos: string[],
): {
  atual: string
  proxima: string | null
  tipo: "novidades" | "correções" | null
  novidades: string[]
  correcoes: string[]
  outras: string[]
}
