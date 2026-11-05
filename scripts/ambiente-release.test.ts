import { expect, it } from "vitest"

import { ambienteRelease } from "./ambiente-release.mjs"

const casa = "C:\\Pessoa Teste"
const projeto = "C:\\Pessoa Teste\\Projetos\\Meu App"
const privacidade = [
  "--remap-path-prefix=C:\\Pessoa Teste=/build/user",
  "--remap-path-prefix=C:/Pessoa Teste=/build/user",
  "--remap-path-prefix=C:\\Pessoa Teste\\Projetos\\Meu App=/build/app",
  "--remap-path-prefix=C:/Pessoa Teste/Projetos/Meu App=/build/app",
  "-C",
  "link-arg=/PDBALTPATH:%_PDB%",
]

it("tira_do_exe_os_caminhos_do_pc_nas_duas_barras", () => {
  expect(ambienteRelease({}, casa, projeto)).toEqual({ CARGO_ENCODED_RUSTFLAGS: privacidade.join("\x1f") })
})

it("mantem_as_flags_codificadas_que_ja_existiam", () => {
  const env = { CARGO_ENCODED_RUSTFLAGS: '--cfg\x1fsabor="com espaço"', RUSTFLAGS: "--cfg=ignorada" }
  expect(ambienteRelease(env, casa, projeto)).toEqual({
    CARGO_ENCODED_RUSTFLAGS: ["--cfg", 'sabor="com espaço"', ...privacidade].join("\x1f"),
  })
})

it("aproveita_rustflags_sem_flags_codificadas", () => {
  expect(ambienteRelease({ RUSTFLAGS: " -C  opt-level=2 " }, casa, projeto)).toEqual({
    CARGO_ENCODED_RUSTFLAGS: ["-C", "opt-level=2", ...privacidade].join("\x1f"),
  })
})
