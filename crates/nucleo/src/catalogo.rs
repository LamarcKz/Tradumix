//! O índice do catálogo (`indice.json`): a lista de fichas publicadas, com os links de download.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::pacote::validar_ficha;
use crate::{ArquivoRemoto, Erro, Ficha};

pub const FORMATO_INDICE: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Indice {
    pub formato: u32,
    pub atualizado: String,
    pub traducoes: Vec<Ficha>,
}

/// Lê e confere o índice. `permitir_http_local` aceita `http://127.0.0.1` (só para testes).
pub fn ler_indice(texto: &str, permitir_http_local: bool) -> Result<Indice, Erro> {
    let invalido = |m: String| Erro::IndiceInvalido(m);
    let indice: Indice = serde_json::from_str(texto).map_err(|e| invalido(format!("JSON inválido: {e}")))?;
    if indice.formato != FORMATO_INDICE {
        return Err(invalido(format!("formato {} não suportado (esperado {FORMATO_INDICE})", indice.formato)));
    }
    let mut ids = HashSet::new();
    for ficha in &indice.traducoes {
        validar_ficha(ficha).map_err(|m| invalido(format!("{}: {m}", ficha.id)))?;
        if !ids.insert(ficha.id.as_str()) {
            return Err(invalido(format!("id repetido: {}", ficha.id)));
        }
        let download = ficha.download.as_ref().ok_or_else(|| invalido(format!("{}: falta o bloco download", ficha.id)))?;
        let arquivos = [Some(&download.patch), download.capa.as_ref(), download.fundo.as_ref()];
        for arquivo in arquivos.into_iter().flatten() {
            conferir_arquivo(arquivo, permitir_http_local).map_err(|m| invalido(format!("{}: {m}", ficha.id)))?;
        }
    }
    Ok(indice)
}

fn conferir_arquivo(arquivo: &ArquivoRemoto, permitir_http_local: bool) -> Result<(), String> {
    let local = permitir_http_local && arquivo.url.starts_with("http://127.0.0.1");
    if !arquivo.url.starts_with("https://") && !local {
        return Err(format!("só links https são aceitos: {}", arquivo.url));
    }
    if arquivo.sha256.len() != 64 || !arquivo.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("sha256 deve ter 64 caracteres hexadecimais".into());
    }
    Ok(())
}
