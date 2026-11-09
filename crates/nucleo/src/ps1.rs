//! Identidade de um jogo de PS1 (`.cue` + `.bin`) e o hash que o RetroAchievements usa.
//!
//! O hash segue o `rc_hash_psx` do rcheevos (`rhash.c`): o executável vem da chave `BOOT` do
//! `SYSTEM.CNF` (ou `PSX.EXE`), o nome entra no MD5 como está no `SYSTEM.CNF` (sem `cdrom:` e sem
//! as barras do início, até o `;`) e, num `PS-X EXE`, só `t_size + 2048` bytes entram, lidos a
//! partir do primeiro setor do executável.

use std::path::Path;

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

use crate::cue::{TipoFaixa, ler_cue};
use crate::iso9660::Setores;
use crate::{Erro, Iso, Regiao, regiao_do_serial, serial_do_executavel};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentidadePs1 {
    pub serial: String,
    pub executavel: String,
    pub regiao: Regiao,
    pub hash_ra: String,
    /// Tamanho do `.bin` da faixa de dados.
    pub tamanho_dados: u64,
    /// Soma de todos os `.bin` do `.cue`.
    pub tamanho_total: u64,
}

pub fn identificar_ps1(cue: &Path) -> Result<IdentidadePs1, Erro> {
    let cue = ler_cue(cue)?;
    let dados = cue.faixa_de_dados().ok_or(Erro::NaoEPs1)?;
    let mut id = identificar_bin_ps1(&dados.arquivo, dados.tipo)?;
    id.tamanho_total = cue
        .arquivos()
        .iter()
        .map(|a| std::fs::metadata(a).map(|m| m.len()))
        .sum::<Result<u64, _>>()?;
    Ok(id)
}

/// Identifica direto pelo `.bin` de dados (para conferir a faixa gerada antes do `.cue` novo).
/// `tamanho_total` sai igual a `tamanho_dados`.
pub fn identificar_bin_ps1(bin: &Path, tipo: TipoFaixa) -> Result<IdentidadePs1, Erro> {
    if tipo == TipoFaixa::Audio {
        return Err(Erro::NaoEPs1);
    }
    let mut iso = match Iso::abrir_com(bin, Setores::do_tipo(tipo)) {
        Ok(iso) => iso,
        Err(Erro::IsoInvalida) => return Err(Erro::NaoEPs1),
        Err(e) => return Err(e),
    };
    let do_boot = match iso.ler_arquivo("SYSTEM.CNF") {
        Ok(cnf) => executavel_do_boot(&cnf),
        Err(Erro::ArquivoNaoEncontrado(_)) => None,
        Err(e) => return Err(e),
    };
    // Como no rcheevos: se o executável do BOOT não existe, tenta o PSX.EXE.
    let mut achado = None;
    if let Some(nome) = do_boot {
        achado = ler_executavel(&mut iso, &nome)?.map(|b| (nome, b));
    }
    if achado.is_none() {
        achado = ler_executavel(&mut iso, "PSX.EXE")?.map(|b| ("PSX.EXE".to_string(), b));
    }
    let (executavel, bytes) = achado.ok_or(Erro::NaoEPs1)?;
    let arquivo = executavel.rsplit(['/', '\\']).next().unwrap_or(&executavel);
    let serial = serial_do_executavel(arquivo);
    Ok(IdentidadePs1 {
        regiao: regiao_do_serial(&serial),
        hash_ra: hash_ra_ps1(&executavel, &bytes),
        serial,
        executavel,
        tamanho_dados: iso.tamanho(),
        tamanho_total: iso.tamanho(),
    })
}

/// Lê o executável, com bytes suficientes para o `t_size` do cabeçalho. `None` se não existe.
fn ler_executavel(iso: &mut Iso, nome: &str) -> Result<Option<Vec<u8>>, Erro> {
    let cabecalho = match iso.ler_arquivo(nome) {
        Ok(b) => b,
        Err(Erro::ArquivoNaoEncontrado(_)) => return Ok(None),
        Err(e) => return Err(e),
    };
    match tamanho_declarado(&cabecalho) {
        Some(t) if t > cabecalho.len() as u64 => iso.ler_a_partir_de(nome, t).map(Some),
        _ => Ok(Some(cabecalho)),
    }
}

/// `t_size + 2048` de um `PS-X EXE`.
fn tamanho_declarado(bytes: &[u8]) -> Option<u64> {
    if bytes.len() < 32 || &bytes[..7] != b"PS-X EX" {
        return None;
    }
    Some(u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]) as u64 + 2048)
}

/// O valor de `BOOT` no `SYSTEM.CNF`, como o rcheevos lê: só no começo da linha, com `BOOT`
/// e `cdrom:` exatos, sem as barras iniciais, até um espaço ou `;`.
fn executavel_do_boot(cnf: &[u8]) -> Option<String> {
    let texto = String::from_utf8_lossy(cnf);
    for linha in texto.split('\n') {
        let Some(resto) = linha.strip_prefix("BOOT") else { continue };
        let resto = resto.trim_start();
        let Some(valor) = resto.strip_prefix('=') else { continue };
        let valor = valor.trim_start();
        let valor = valor.strip_prefix("cdrom:").unwrap_or(valor);
        let valor = valor.trim_start_matches('\\');
        let fim = valor.find(|c: char| c.is_whitespace() || c == ';').unwrap_or(valor.len());
        // O rcheevos para na primeira linha BOOT que achar, com ou sem nome.
        return (fim > 0).then(|| valor[..fim].to_string());
    }
    None
}

/// MD5 do nome do executável seguido dos bytes; num `PS-X EXE`, só `t_size + 2048` bytes.
pub fn hash_ra_ps1(executavel: &str, bytes: &[u8]) -> String {
    let fim = tamanho_declarado(bytes).map(|t| (t as usize).min(bytes.len())).unwrap_or(bytes.len());
    let mut md5 = Md5::new();
    md5.update(executavel.as_bytes());
    md5.update(&bytes[..fim]);
    md5.finalize().iter().map(|b| format!("{b:02x}")).collect()
}
