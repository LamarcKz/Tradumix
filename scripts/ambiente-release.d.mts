export function ambienteRelease(
  env: Record<string, string | undefined>,
  pastaDoUsuario: string,
  pastaDoProjeto: string,
): { CARGO_ENCODED_RUSTFLAGS: string }
