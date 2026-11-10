//! PC: a cópia da pessoa é a pasta do jogo instalado. A tradução é um `.zip` com arquivos que vão
//! para dentro dessa pasta; os que já existem são guardados para "Remover tradução".

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{Alvo, InfoPlataforma};
use crate::instalar_pc::no_jogo;
use crate::rede::sha256_arquivo;
use crate::{Erro, Ficha, Instalacao};

pub(crate) const INFO: InfoPlataforma = InfoPlataforma {
    id: "pc",
    nome: "PC",
    sigla: "PC",
    arquivo: "pasta do jogo",
    extensoes: &[],
    tem_ra: false,
    escolhe_pasta: true,
    proporcao_capa: "2/3",
};

/// Capa vertical da biblioteca da Steam (600×900), pelo id do jogo na loja.
pub const ACERVO_CAPAS_STEAM: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps/{id}/library_600x900.jpg";

/// Um arquivo da pasta do jogo que identifica a versão testada.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenciaPc {
    pub caminho: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrigemPc {
    /// Loja de onde vem a versão testada ("Steam", "GOG"…).
    pub loja: String,
    /// Versão do jogo testada, como a loja mostra.
    pub versao: String,
    pub referencias: Vec<ReferenciaPc>,
    /// Id do jogo na Steam: dá a capa do card. Opcional (jogos de outras lojas trazem `imagens.capa`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steam_app_id: Option<u64>,
}

/// O resultado do PC não tem campos: a instalação é conferida arquivo por arquivo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ResultadoPc {}

pub(crate) fn ler(ficha: &Ficha) -> Result<Alvo, String> {
    let origem: OrigemPc = serde_json::from_value(ficha.origem.clone()).map_err(|e| format!("origem inválida: {e}"))?;
    let _: ResultadoPc = serde_json::from_value(ficha.resultado.clone()).map_err(|e| format!("resultado inválido: {e}"))?;
    if origem.referencias.is_empty() {
        return Err("origem.referencias precisa ter ao menos um arquivo do jogo".into());
    }
    for r in &origem.referencias {
        conferir_caminho(&r.caminho)?;
        conferir_sha(&r.sha256, &r.caminho)?;
    }
    if let Instalacao::PastaDoJogo { arquivos, .. } = &ficha.instalacao {
        if arquivos.is_empty() {
            return Err("instalacao.arquivos está vazia".into());
        }
        let mut vistos = std::collections::HashSet::new();
        for a in arquivos {
            conferir_caminho(&a.caminho)?;
            conferir_sha(&a.sha256, &a.caminho)?;
            // No Windows, `a.txt` e `A.TXT` são o mesmo arquivo.
            if !vistos.insert(a.caminho.to_lowercase()) {
                return Err(format!("caminho repetido (o Windows não diferencia maiúsculas): {}", a.caminho));
            }
        }
    }
    Ok(Alvo::Pc { origem })
}

/// Confere pelas referências se a pasta é do jogo. Nenhuma referência na pasta: pasta errada.
/// Alguma faltando ou diferente: outra versão do jogo, instala com aviso.
pub(crate) fn conferir(ficha: &Ficha, origem: &OrigemPc, pasta: &Path) -> Result<Option<String>, Erro> {
    let errada = || Erro::PastaDoJogoErrada { jogo: ficha.jogo.nome.clone() };
    if !pasta.is_dir() {
        return Err(errada());
    }
    let mut existem = 0;
    let mut iguais = 0;
    for r in &origem.referencias {
        let caminho = no_jogo(pasta, &r.caminho);
        if caminho.is_file() {
            existem += 1;
            if sha256_arquivo(&caminho).is_some_and(|s| s.eq_ignore_ascii_case(&r.sha256)) {
                iguais += 1;
            }
        }
    }
    if existem == 0 {
        return Err(errada());
    }
    if iguais < origem.referencias.len() {
        return Ok(Some(format!(
            "Esta pasta tem uma versão do jogo diferente da testada ({}). A tradução pode não funcionar direito.",
            origem.versao
        )));
    }
    Ok(None)
}

fn conferir_sha(sha: &str, caminho: &str) -> Result<(), String> {
    if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("o sha256 de {caminho} deve ter 64 caracteres hexadecimais"));
    }
    Ok(())
}

/// Nomes que o Windows reserva, com ou sem extensão.
const RESERVADOS: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4",
    "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Um caminho relativo à pasta do jogo, com `/`, que não sai dela nem usa nome proibido no Windows.
pub(crate) fn conferir_caminho(caminho: &str) -> Result<(), String> {
    let ruim = |motivo: &str| Err(format!("caminho inválido ({motivo}): {caminho:?}"));
    if caminho.is_empty() {
        return ruim("vazio");
    }
    if caminho.contains('\\') {
        return ruim("use / para separar pastas");
    }
    if caminho.starts_with('/') {
        return ruim("tem de ser relativo à pasta do jogo");
    }
    // `.tradumix/` guarda o registro e os originais: a tradução não pode gravar ali.
    if caminho.split('/').next().is_some_and(|p| p.eq_ignore_ascii_case(".tradumix")) {
        return ruim("a pasta .tradumix é do app");
    }
    for parte in caminho.split('/') {
        if parte.is_empty() {
            return ruim("pasta sem nome");
        }
        if parte == "." || parte == ".." {
            return ruim("não pode usar . nem ..");
        }
        if parte.chars().any(|c| c.is_control() || ":*?\"<>|".contains(c)) {
            return ruim("caractere proibido");
        }
        if parte.ends_with('.') || parte.ends_with(' ') {
            return ruim("nome terminado em ponto ou espaço");
        }
        let base = parte.split('.').next().unwrap_or(parte).trim_end();
        if RESERVADOS.iter().any(|r| base.eq_ignore_ascii_case(r)) {
            return ruim("nome reservado do Windows");
        }
    }
    Ok(())
}
