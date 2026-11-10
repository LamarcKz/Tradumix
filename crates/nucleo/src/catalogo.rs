//! O índice do catálogo (`indice-v2.json`): a lista de fichas publicadas, com os links de download.
//!
//! Cada ficha é lida sozinha. A que esta versão do app não sabe instalar (plataforma ou instalação
//! desconhecida) ou que não passa nas regras fica de fora, e as outras continuam valendo.

use std::collections::HashSet;

use serde::Deserialize;

use crate::pacote::validar_ficha;
use crate::{ArquivoBaixavel, Erro, Ficha};

pub const FORMATO_INDICE: u32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct Indice {
    pub formato: u32,
    pub atualizado: String,
    pub traducoes: Vec<Ficha>,
    /// As fichas que ficaram de fora, como `"<id ou posição>: <motivo>"`.
    pub ignoradas: Vec<String>,
}

#[derive(Deserialize)]
struct IndiceBruto {
    formato: u32,
    #[serde(default)]
    atualizado: String,
    traducoes: Vec<serde_json::Value>,
}

/// Lê o índice. Só recusa o índice inteiro se o JSON não abrir ou o formato não for o desta versão.
/// `permitir_http_local` aceita `http://127.0.0.1` (só para testes).
pub fn ler_indice(texto: &str, permitir_http_local: bool) -> Result<Indice, Erro> {
    let bruto: IndiceBruto = serde_json::from_str(texto).map_err(|e| Erro::IndiceInvalido(format!("JSON inválido: {e}")))?;
    if bruto.formato != FORMATO_INDICE {
        return Err(Erro::IndiceInvalido(format!("formato {} não suportado (esperado {FORMATO_INDICE})", bruto.formato)));
    }
    let mut indice = Indice { formato: bruto.formato, atualizado: bruto.atualizado, traducoes: Vec::new(), ignoradas: Vec::new() };
    let mut ids = HashSet::new();
    for (posicao, valor) in bruto.traducoes.into_iter().enumerate() {
        let nome = valor.get("id").and_then(|v| v.as_str()).map(str::to_string).unwrap_or_else(|| format!("#{}", posicao + 1));
        match ler_ficha(valor, permitir_http_local) {
            Ok(ficha) if ids.contains(&ficha.id) => indice.ignoradas.push(format!("{nome}: id repetido")),
            Ok(ficha) => {
                ids.insert(ficha.id.clone());
                indice.traducoes.push(ficha);
            }
            Err(motivo) => indice.ignoradas.push(format!("{nome}: {motivo}")),
        }
    }
    Ok(indice)
}

fn ler_ficha(valor: serde_json::Value, permitir_http_local: bool) -> Result<Ficha, String> {
    let ficha: Ficha = serde_json::from_value(valor).map_err(|e| format!("ficha inválida: {e}"))?;
    validar_ficha(&ficha)?;
    for arquivo in ficha.arquivos() {
        conferir_arquivo(arquivo, permitir_http_local)?;
    }
    Ok(ficha)
}

fn conferir_arquivo(arquivo: &ArquivoBaixavel, permitir_http_local: bool) -> Result<(), String> {
    let local = permitir_http_local && arquivo.url.starts_with("http://127.0.0.1");
    if !arquivo.url.starts_with("https://") && !local {
        return Err(format!("só links https são aceitos: {}", arquivo.url));
    }
    if arquivo.sha256.len() != 64 || !arquivo.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("sha256 deve ter 64 caracteres hexadecimais".into());
    }
    Ok(())
}
