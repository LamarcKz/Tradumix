//! Identidade de um jogo de PS2 e o hash que o RetroAchievements usa.

use std::path::Path;

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

use crate::{Erro, Iso};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Regiao {
    Usa,
    Europa,
    Japao,
    Desconhecida,
}

impl Regiao {
    /// Nome mostrado ao usuário e usado no nome da ISO gerada.
    pub fn nome(&self) -> &str {
        match self {
            Regiao::Usa => "USA",
            Regiao::Europa => "Europa",
            Regiao::Japao => "Japão",
            Regiao::Desconhecida => "Desconhecida",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentidadePs2 {
    pub serial: String,
    pub executavel: String,
    pub regiao: Regiao,
    pub hash_ra: String,
    pub tamanho: u64,
}

/// Lê o `SYSTEM.CNF` e o executável para dizer que jogo é a ISO.
pub fn identificar(caminho: &Path) -> Result<IdentidadePs2, Erro> {
    let mut iso = Iso::abrir(caminho)?;
    let cnf = match iso.ler_arquivo("SYSTEM.CNF") {
        Ok(c) => c,
        Err(Erro::ArquivoNaoEncontrado(_)) => return Err(Erro::NaoEPs2),
        Err(e) => return Err(e),
    };
    let caminho_exe = caminho_do_boot2(&String::from_utf8_lossy(&cnf)).ok_or(Erro::NaoEPs2)?;
    let bytes = iso.ler_arquivo(&caminho_exe)?;
    let executavel = caminho_exe.to_uppercase();
    let nome = executavel.rsplit(['/', '\\']).next().unwrap_or(&executavel).to_string();
    let serial = serial_do_executavel(&nome);
    Ok(IdentidadePs2 {
        regiao: regiao_do_serial(&serial),
        hash_ra: hash_ra(&executavel, &bytes),
        serial,
        executavel,
        tamanho: iso.tamanho(),
    })
}

/// Caminho do executável na linha `BOOT2`, sem `cdrom0:\` e sem `;1`.
fn caminho_do_boot2(cnf: &str) -> Option<String> {
    for linha in cnf.lines() {
        let Some((chave, valor)) = linha.split_once('=') else { continue };
        if !chave.trim().eq_ignore_ascii_case("BOOT2") {
            continue;
        }
        let mut v = valor.trim();
        if v.len() >= 7 && v[..7].eq_ignore_ascii_case("cdrom0:") {
            v = &v[7..];
        }
        let v = v.trim_start_matches(['\\', '/']);
        let v = v.split(';').next().unwrap_or("").trim();
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

/// MD5 do nome do executável seguido dos bytes dele, em hex minúsculo.
pub fn hash_ra(executavel: &str, bytes: &[u8]) -> String {
    let mut md5 = Md5::new();
    md5.update(executavel.as_bytes());
    md5.update(bytes);
    md5.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// `SCUS_972.65` → `SCUS-97265`.
pub fn serial_do_executavel(executavel: &str) -> String {
    let maiusculo = executavel.to_uppercase();
    match maiusculo.split_once('_') {
        Some((prefixo, resto)) => format!("{prefixo}-{}", resto.replace('.', "")),
        None => maiusculo,
    }
}

pub fn regiao_do_serial(serial: &str) -> Regiao {
    let prefixo = serial.get(..4).unwrap_or("").to_uppercase();
    match prefixo.as_str() {
        "SCUS" | "SLUS" | "PBPX" => Regiao::Usa,
        "SCES" | "SLES" | "SCED" => Regiao::Europa,
        "SCPS" | "SLPS" | "SLPM" | "SCAJ" => Regiao::Japao,
        _ => Regiao::Desconhecida,
    }
}
