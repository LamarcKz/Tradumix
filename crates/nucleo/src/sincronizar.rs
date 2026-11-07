//! Traz o catálogo para o cache: cada tradução aceita vira uma pasta de pacote (ficha + capa + fundo).
//! O que não está entre as aceitas (saiu do índice ou ficou de fora) é apagado do cache.
//! O patch fica para a hora de traduzir.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::rede::{baixar_conferido, sha256_arquivo};
use crate::{Ficha, NOME_CAPA_ACERVO, Rede, ler_alvo, ler_indice};

const LIMITE_INDICE: u64 = 2 * 1024 * 1024;
const LIMITE_IMAGEM: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sincronizacao {
    pub offline: bool,
    pub atualizado_em: Option<String>,
    pub aviso: Option<String>,
}

/// De onde vem a capa das traduções que não têm capa própria.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capas {
    /// Não busca capa.
    Nao,
    /// O acervo de cada plataforma (o normal no app).
    Acervos,
    /// Um modelo de endereço no lugar do acervo (testes); a plataforma preenche o modelo.
    Modelo(String),
}

pub fn sincronizar(rede: &dyn Rede, url_indice: &str, cache: &Path, permitir_http_local: bool, capas: Capas) -> Sincronizacao {
    let ultima = fs::read_to_string(cache.join("sincronizado.txt")).ok().map(|t| t.trim().to_string());
    let mut texto = Vec::new();
    if rede.baixar(url_indice, &mut texto, LIMITE_INDICE, &mut |_| {}).is_err() {
        let aviso = match &ultima {
            Some(data) => format!("Sem conexão: mostrando o catálogo baixado em {}.", data_br(data)),
            None => "Sem conexão: o catálogo ainda não foi baixado.".to_string(),
        };
        return Sincronizacao { offline: true, atualizado_em: ultima, aviso: Some(aviso) };
    }
    let indice = match std::str::from_utf8(&texto).ok().map(|t| ler_indice(t, permitir_http_local)) {
        Some(Ok(i)) => i,
        _ => {
            return Sincronizacao {
                offline: false,
                atualizado_em: ultima,
                aviso: Some("O catálogo baixado veio com erro; mostrando o anterior.".into()),
            };
        }
    };

    let _ = fs::create_dir_all(cache);
    let ids: HashSet<&str> = indice.traducoes.iter().map(|f| f.id.as_str()).collect();
    for ficha in &indice.traducoes {
        gravar_traducao(rede, ficha, &cache.join(&ficha.id), &capas);
    }
    remover_saidas(cache, &ids);
    let hoje = chrono::Local::now().format("%Y-%m-%d").to_string();
    let _ = fs::write(cache.join("indice.json"), &texto);
    let _ = fs::write(cache.join("sincronizado.txt"), &hoje);
    Sincronizacao { offline: false, atualizado_em: Some(hoje), aviso: None }
}

fn gravar_traducao(rede: &dyn Rede, ficha: &Ficha, pasta: &Path, capas: &Capas) {
    if fs::create_dir_all(pasta).is_err() {
        return;
    }
    // Versão com outro patch: o patch baixado antes não serve mais.
    let anterior: Option<Ficha> = fs::read_to_string(pasta.join("ficha.json")).ok().and_then(|t| serde_json::from_str(&t).ok());
    let sha_antigo = anterior.as_ref().map(|f| f.patch().sha256.to_lowercase());
    if sha_antigo.as_deref() != Some(ficha.patch().sha256.to_lowercase().as_str()) {
        let _ = fs::remove_file(pasta.join(&ficha.patch().nome));
    }
    if let Ok(texto) = serde_json::to_string_pretty(ficha) {
        let _ = fs::write(pasta.join("ficha.json"), texto);
    }
    // Sem capa própria: a capa vem do acervo da plataforma e fica na pasta como capa.jpg.
    let imagens = ficha.imagens.clone().unwrap_or_default();
    if imagens.capa.is_none() {
        let url = match (capas, ler_alvo(ficha)) {
            (Capas::Acervos, Ok(alvo)) => alvo.url_capa(None),
            (Capas::Modelo(modelo), Ok(alvo)) => alvo.url_capa(Some(modelo)),
            _ => None,
        };
        if let Some(url) = url {
            capa_do_acervo(rede, &url, &pasta.join(NOME_CAPA_ACERVO));
        }
    }
    for remoto in imagens.capa.iter().chain(imagens.fundo.iter()) {
        let destino = pasta.join(&remoto.nome);
        if sha256_arquivo(&destino).is_some_and(|s| s.eq_ignore_ascii_case(&remoto.sha256)) {
            continue;
        }
        // Imagem que não baixa ou não confere fica de fora; o resto continua.
        if baixar_conferido(rede, remoto, &destino, LIMITE_IMAGEM, &mut |_| {}, None).is_err() {
            let _ = fs::remove_file(&destino);
        }
    }
}

/// Baixa a capa uma vez só; aceita apenas JPEG ou PNG de verdade (o acervo responde HTML em alguns erros).
fn capa_do_acervo(rede: &dyn Rede, url: &str, destino: &Path) -> bool {
    if destino.is_file() {
        return true;
    }
    let mut bytes = Vec::new();
    if rede.baixar(url, &mut bytes, LIMITE_IMAGEM, &mut |_| {}).is_err() {
        return false;
    }
    let imagem = bytes.starts_with(&[0xFF, 0xD8, 0xFF]) || bytes.starts_with(&[0x89, b'P', b'N', b'G']);
    imagem && fs::write(destino, &bytes).is_ok()
}

/// Apaga do cache as traduções que saíram do índice.
fn remover_saidas(cache: &Path, ids: &HashSet<&str>) {
    let Ok(entradas) = fs::read_dir(cache) else { return };
    for entrada in entradas.flatten() {
        let caminho = entrada.path();
        let nome = entrada.file_name().to_string_lossy().into_owned();
        if caminho.join("ficha.json").is_file() && !ids.contains(nome.as_str()) {
            let _ = fs::remove_dir_all(&caminho);
        }
    }
}

/// `2026-11-05` → `05/11/2026`.
fn data_br(data: &str) -> String {
    let partes: Vec<&str> = data.split('-').collect();
    if partes.len() == 3 { format!("{}/{}/{}", partes[2], partes[1], partes[0]) } else { data.to_string() }
}
