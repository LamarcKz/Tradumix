//! PlayStation: a cópia da pessoa é um `.cue` com os `.bin` (Redump, um `.bin` por faixa). O patch
//! vale para o `.bin` da faixa de dados; as faixas de áudio são copiadas e o `.cue` novo é escrito
//! com os nomes novos.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{Alvo, InfoPlataforma, nome_de_arquivo_seguro};
use crate::cue::{Cue, TipoFaixa, escrever_cue, ler_cue};
use crate::{Erro, Ficha, IdentidadePs1, Regiao, identificar_bin_ps1, identificar_ps1};

pub(crate) const INFO: InfoPlataforma = InfoPlataforma {
    id: "ps1",
    nome: "PlayStation",
    sigla: "PS1",
    arquivo: "imagem (.cue)",
    extensoes: &["cue"],
    tem_ra: true,
};

/// Acervo público de capas de PS1 (o mesmo autor do acervo de PS2), endereçado pelo serial.
pub const ACERVO_CAPAS: &str = "https://raw.githubusercontent.com/xlenore/psx-covers/main/covers/default/{serial}.jpg";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrigemPs1 {
    pub serial: String,
    pub regiao: Regiao,
    pub hash_ra: String,
    /// Tamanho do `.bin` da faixa de dados.
    pub tamanho: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultadoPs1 {
    pub hash_ra: String,
    /// Tamanho do `.bin` de dados gerado.
    pub tamanho: u64,
}

pub(crate) fn ler(ficha: &Ficha) -> Result<Alvo, String> {
    let origem: OrigemPs1 = serde_json::from_value(ficha.origem.clone()).map_err(|e| format!("origem inválida: {e}"))?;
    let resultado: ResultadoPs1 = serde_json::from_value(ficha.resultado.clone()).map_err(|e| format!("resultado inválido: {e}"))?;
    super::conferir_hashes(&origem.hash_ra, &resultado.hash_ra)?;
    if origem.serial.trim().is_empty() {
        return Err("o campo origem.serial está vazio".into());
    }
    Ok(Alvo::Ps1 { origem, resultado })
}

pub(crate) fn conferir(ficha: &Ficha, origem: &OrigemPs1, cue: &Path) -> Result<(), Erro> {
    let id: IdentidadePs1 = identificar_ps1(cue)?;
    let esperado_nome = format!("{} ({})", ficha.jogo.nome, origem.regiao.nome());
    if !id.serial.eq_ignore_ascii_case(&origem.serial) {
        return Err(Erro::JogoErrado { esperado_nome, esperado_serial: origem.serial.clone(), encontrado_serial: id.serial });
    }
    if !id.hash_ra.eq_ignore_ascii_case(&origem.hash_ra) {
        return Err(Erro::VersaoDiferente { esperado_nome, serial: origem.serial.clone() });
    }
    if id.tamanho_dados != origem.tamanho {
        return Err(Erro::TamanhoDiferente { esperado: origem.tamanho, encontrado: id.tamanho_dados });
    }
    Ok(())
}

/// Mesma pasta da original: `<jogo> (<Região>) <IDIOMA> v<versão>.cue`.
pub(crate) fn nome_saida(ficha: &Ficha, origem: &OrigemPs1, cue: &Path) -> PathBuf {
    let pasta = cue.parent().unwrap_or(Path::new(""));
    let nome = format!("{} ({}) {} v{}", ficha.jogo.nome, origem.regiao.nome(), ficha.idioma.to_uppercase(), ficha.versao);
    pasta.join(format!("{}.cue", nome_de_arquivo_seguro(&nome)))
}

/// Cada `.bin` da original e o nome que ele ganha na cópia traduzida, na ordem do `.cue`.
/// Com uma faixa só, `<saída>.bin`; com várias, `<saída> (Faixa N).bin`.
fn planejar(cue: &Cue, saida: &Path) -> Vec<(PathBuf, PathBuf)> {
    let base = saida.with_extension("");
    let base_nome = base.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let pasta = saida.parent().unwrap_or(Path::new(""));
    let arquivos = cue.arquivos();
    let varias = arquivos.len() > 1;
    arquivos
        .into_iter()
        .map(|original| {
            let numero = cue.faixas.iter().find(|f| f.arquivo == original).map(|f| f.numero).unwrap_or(1);
            let nome = if varias { format!("{base_nome} (Faixa {numero}).bin") } else { format!("{base_nome}.bin") };
            (original, pasta.join(nome))
        })
        .collect()
}

fn dados(cue: &Cue) -> Result<PathBuf, Erro> {
    cue.faixa_de_dados().map(|f| f.arquivo.clone()).ok_or(Erro::NaoEPs1)
}

/// O `.cue` e todos os `.bin` que a instalação cria.
pub(crate) fn arquivos_saida(original: &Path, saida: &Path) -> Result<Vec<PathBuf>, Erro> {
    let cue = ler_cue(original)?;
    let mut lista = vec![saida.to_path_buf()];
    lista.extend(planejar(&cue, saida).into_iter().map(|(_, novo)| novo));
    Ok(lista)
}

pub(crate) fn entrada_patch(original: &Path) -> Result<PathBuf, Erro> {
    dados(&ler_cue(original)?)
}

pub(crate) fn saida_patch(original: &Path, saida: &Path) -> Result<PathBuf, Erro> {
    let cue = ler_cue(original)?;
    let dados = dados(&cue)?;
    planejar(&cue, saida).into_iter().find(|(o, _)| *o == dados).map(|(_, novo)| novo).ok_or(Erro::NaoEPs1)
}

/// Copia as faixas que não são a de dados e grava o `.cue` novo.
pub(crate) fn completar_saida(original: &Path, saida: &Path) -> Result<(), Erro> {
    let cue = ler_cue(original)?;
    let dados = dados(&cue)?;
    let plano = planejar(&cue, saida);
    for (de, para) in &plano {
        if *de != dados {
            fs::copy(de, para)?;
        }
    }
    let nomes: Vec<String> = plano.iter().map(|(_, novo)| novo.file_name().unwrap_or_default().to_string_lossy().into_owned()).collect();
    fs::write(saida, escrever_cue(&cue, &nomes))?;
    Ok(())
}

/// Soma das faixas copiadas (as que não recebem o patch).
pub(crate) fn tamanho_extra(original: &Path) -> u64 {
    let Ok(cue) = ler_cue(original) else { return 0 };
    let dados = cue.faixa_de_dados().map(|f| f.arquivo.clone());
    cue.arquivos().iter().filter(|a| Some(*a) != dados.as_ref()).filter_map(|a| fs::metadata(a).ok()).map(|m| m.len()).sum()
}

/// Hash do RetroAchievements do `.bin` de dados gerado, lido com o tipo da faixa da original.
pub(crate) fn verificar_resultado(original: &Path, gerado: &Path) -> Result<Option<String>, Erro> {
    let cue = ler_cue(original)?;
    let tipo = cue.faixa_de_dados().map(|f| f.tipo).unwrap_or(TipoFaixa::Mode2_2352);
    Ok(Some(identificar_bin_ps1(gerado, tipo)?.hash_ra))
}
