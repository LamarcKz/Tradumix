// Diz qual deve ser a próxima versão, olhando os commits desde a última (Conventional Commits), como no
// PlayTrove:
// - algum "feat:" (novidade): sobe o número do meio (0.1.0 → 0.2.0). Só lançar com o OK do autor;
// - só "fix:" (correções): sobe o último número (0.2.0 → 0.2.1);
// - só outros tipos (docs, chore, refactor, ci...): nada muda para quem usa, não precisa de versão.
// Também monta um rascunho das notas. Uso: npm run versao
import { execFileSync } from "node:child_process"
import { pathToFileURL } from "node:url"

/** A próxima versão a partir da última tag ("v0.1.0") e do assunto de cada commit depois dela. */
export function proximaVersao(ultimaTag, assuntos) {
  const [maior, meio, ultimo] = ultimaTag.replace(/^v/, "").split(".").map(Number)
  const novidades = []
  const correcoes = []
  const outras = []
  for (const assunto of assuntos) {
    // "feat(app): tela Sobre" → tipo "feat", texto "Tela Sobre". O "!" marca mudança que quebra algo.
    const m = /^(\w+)(\([^)]*\))?(!)?:\s*(.+)$/.exec(assunto)
    const tipo = m?.[1]
    const texto = m ? m[4].charAt(0).toUpperCase() + m[4].slice(1) : assunto
    if (tipo === "feat" || m?.[3]) novidades.push(texto)
    else if (tipo === "fix") correcoes.push(texto)
    else outras.push(assunto)
  }
  let proxima = null
  let tipo = null
  if (novidades.length > 0) {
    proxima = `${maior}.${meio + 1}.0`
    tipo = "novidades"
  } else if (correcoes.length > 0) {
    proxima = `${maior}.${meio}.${ultimo + 1}`
    tipo = "correções"
  }
  return { atual: `${maior}.${meio}.${ultimo}`, proxima, tipo, novidades, correcoes, outras }
}

function principal() {
  const git = (...args) => execFileSync("git", args, { encoding: "utf8" }).trim()
  let ultimaTag
  try {
    ultimaTag = git("describe", "--tags", "--abbrev=0", "--match", "v[0-9]*")
  } catch {
    console.error("Nenhuma versão lançada ainda (nenhuma tag vX.Y.Z).")
    process.exit(1)
  }
  const assuntos = git("log", `${ultimaTag}..HEAD`, "--no-merges", "--format=%s").split("\n").filter(Boolean)
  const r = proximaVersao(ultimaTag, assuntos)
  console.log(`Última versão: ${r.atual} (tag ${ultimaTag})`)
  console.log(`Desde então: ${r.novidades.length} novidade(s), ${r.correcoes.length} correção(ões), ${r.outras.length} outra(s).\n`)
  if (!r.proxima) {
    console.log(`→ Nenhuma novidade nem correção desde a ${ultimaTag}: não precisa de versão nova.`)
    return
  }
  console.log(`→ Próxima versão: ${r.proxima} (${r.tipo})\n`)
  console.log("Rascunho das notas (revisar antes de publicar):\n")
  if (r.novidades.length) console.log(["## Novidades", ...r.novidades.map((t) => `- ${t}`), ""].join("\n"))
  if (r.correcoes.length) console.log(["## Correções", ...r.correcoes.map((t) => `- ${t}`), ""].join("\n"))
}

// Só roda quando chamado direto (npm run versao), não quando os testes importam.
if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) principal()
