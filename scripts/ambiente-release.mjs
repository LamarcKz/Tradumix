// Opções da compilação de release (copiado do PlayTrove): tira do Tradumix.exe os caminhos do PC de
// quem compila (pasta do usuário e do projeto), que o Rust grava nas mensagens de erro e no .pdb.
// Não muda o ambiente do processo que chama: devolve só a variável para passar ao build.
export function ambienteRelease(env, pastaDoUsuario, pastaDoProjeto) {
  const codificadas = env.CARGO_ENCODED_RUSTFLAGS
  let flags
  if (codificadas !== undefined) {
    flags = codificadas ? codificadas.split("\x1f") : []
  } else if (env.RUSTFLAGS !== undefined) {
    // O Cargo divide RUSTFLAGS por espaços e não interpreta aspas.
    flags = env.RUSTFLAGS.split(" ").map((f) => f.trim()).filter(Boolean)
  } else {
    const doAlvo = env.CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS?.trim()
    flags = (doAlvo || env.CARGO_BUILD_RUSTFLAGS || "").split(/\s+/).filter(Boolean)
  }
  // Vale o último prefixo que casa: o do projeto vem depois do do usuário.
  for (const [pasta, troca] of [[pastaDoUsuario, "/build/user"], [pastaDoProjeto, "/build/app"]]) {
    for (const prefixo of new Set([pasta.replaceAll("/", "\\"), pasta.replaceAll("\\", "/")])) {
      flags.push(`--remap-path-prefix=${prefixo}=${troca}`)
    }
  }
  flags.push("-C", "link-arg=/PDBALTPATH:%_PDB%")
  return { CARGO_ENCODED_RUSTFLAGS: flags.join("\x1f") }
}
