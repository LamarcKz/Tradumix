//! PlayStation 2: a cópia da pessoa é uma ISO; o jogo é reconhecido pelo `SYSTEM.CNF` e pelo
//! executável, e o hash do RetroAchievements sai do executável.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{Alvo, InfoPlataforma, nome_de_arquivo_seguro};
use crate::{Erro, Ficha, Regiao, identificar};

pub(crate) const INFO: InfoPlataforma = InfoPlataforma {
    id: "ps2",
    nome: "PlayStation 2",
    sigla: "PS2",
    arquivo: "ISO",
    extensoes: &["iso"],
    tem_ra: true,
};

/// Acervo público de capas de PS2 usado pelo PCSX2, endereçado pelo serial do jogo.
pub const ACERVO_CAPAS: &str = "https://raw.githubusercontent.com/xlenore/ps2-covers/main/covers/default/{serial}.jpg";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrigemPs2 {
    pub serial: String,
    pub regiao: Regiao,
    pub hash_ra: String,
    pub tamanho: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultadoPs2 {
    pub hash_ra: String,
    pub tamanho: u64,
}

pub(crate) fn ler(ficha: &Ficha) -> Result<Alvo, String> {
    let origem: OrigemPs2 = serde_json::from_value(ficha.origem.clone()).map_err(|e| format!("origem inválida: {e}"))?;
    let resultado: ResultadoPs2 = serde_json::from_value(ficha.resultado.clone()).map_err(|e| format!("resultado inválido: {e}"))?;
    super::conferir_hashes(&origem.hash_ra, &resultado.hash_ra)?;
    if origem.serial.trim().is_empty() {
        return Err("o campo origem.serial está vazio".into());
    }
    Ok(Alvo::Ps2 { origem, resultado })
}

/// Confere se a ISO é a que a tradução espera, sem ler a imagem inteira.
pub(crate) fn conferir(ficha: &Ficha, origem: &OrigemPs2, iso: &Path) -> Result<(), Erro> {
    let id = identificar(iso)?;
    let esperado_nome = format!("{} ({})", ficha.jogo.nome, origem.regiao.nome());
    if !id.serial.eq_ignore_ascii_case(&origem.serial) {
        return Err(Erro::JogoErrado { esperado_nome, esperado_serial: origem.serial.clone(), encontrado_serial: id.serial });
    }
    if !id.hash_ra.eq_ignore_ascii_case(&origem.hash_ra) {
        return Err(Erro::VersaoDiferente { esperado_nome, serial: origem.serial.clone() });
    }
    if id.tamanho != origem.tamanho {
        return Err(Erro::TamanhoDiferente { esperado: origem.tamanho, encontrado: id.tamanho });
    }
    Ok(())
}

/// Mesma pasta da original: `<jogo> (<Região>) <IDIOMA> v<versão>.iso`.
pub(crate) fn nome_saida(ficha: &Ficha, origem: &OrigemPs2, iso: &Path) -> PathBuf {
    let pasta = iso.parent().unwrap_or(Path::new(""));
    let nome = format!("{} ({}) {} v{}", ficha.jogo.nome, origem.regiao.nome(), ficha.idioma.to_uppercase(), ficha.versao);
    pasta.join(format!("{}.iso", nome_de_arquivo_seguro(&nome)))
}
