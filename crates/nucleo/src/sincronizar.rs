//! Traz o catálogo para o cache: cada tradução vira uma pasta de pacote (ficha + capa + fundo).
//! O patch fica para a hora de traduzir.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::rede::{baixar_conferido, sha256_arquivo};
use crate::{Ficha, Rede, ler_indice};

const LIMITE_INDICE: u64 = 2 * 1024 * 1024;
const LIMITE_IMAGEM: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sincronizacao {
    pub offline: bool,
    pub atualizado_em: Option<String>,
    pub aviso: Option<String>,
}

/// Acervo público de capas de PS2 usado pelo PCSX2, endereçado pelo serial do jogo.
pub const ACERVO_CAPAS_PS2: &str = "https://raw.githubusercontent.com/xlenore/ps2-covers/main/covers/default/{serial}.jpg";

/// `url_capas`: modelo com `{serial}` para buscar a capa das traduções que não têm capa própria
/// (normalmente `ACERVO_CAPAS_PS2`); `None` não busca.
pub fn sincronizar(rede: &dyn Rede, url_indice: &str, cache: &Path, permitir_http_local: bool, url_capas: Option<&str>) -> Sincronizacao {
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
        gravar_traducao(rede, ficha, &cache.join(&ficha.id), url_capas);
    }
    remover_saidas(cache, &ids);
    let hoje = chrono::Local::now().format("%Y-%m-%d").to_string();
    let _ = fs::write(cache.join("indice.json"), &texto);
    let _ = fs::write(cache.join("sincronizado.txt"), &hoje);
    Sincronizacao { offline: false, atualizado_em: Some(hoje), aviso: None }
}

fn gravar_traducao(rede: &dyn Rede, ficha: &Ficha, pasta: &Path, url_capas: Option<&str>) {
    if fs::create_dir_all(pasta).is_err() {
        return;
    }
    // Versão com outro patch: o patch baixado antes não serve mais.
    let anterior: Option<Ficha> = fs::read_to_string(pasta.join("ficha.json")).ok().and_then(|t| serde_json::from_str(&t).ok());
    let sha_antigo = anterior.as_ref().and_then(|f| f.download.as_ref()).map(|d| d.patch.sha256.to_lowercase());
    let sha_novo = ficha.download.as_ref().map(|d| d.patch.sha256.to_lowercase());
    if sha_antigo != sha_novo {
        let _ = fs::remove_file(pasta.join(&ficha.arquivos.patch));
    }
    // Sem capa própria: a capa vem do acervo pelo serial e fica no cache como capa.jpg.
    let mut ficha = ficha.clone();
    if ficha.arquivos.capa.is_none() {
        if let Some(modelo) = url_capas {
            if capa_do_acervo(rede, modelo, &ficha.origem.serial, &pasta.join("capa.jpg")) {
                ficha.arquivos.capa = Some("capa.jpg".into());
            }
        }
    }
    let ficha = &ficha;
    if let Ok(texto) = serde_json::to_string_pretty(ficha) {
        let _ = fs::write(pasta.join("ficha.json"), texto);
    }
    let Some(download) = &ficha.download else { return };
    let imagens = [(download.capa.as_ref(), ficha.arquivos.capa.as_ref()), (download.fundo.as_ref(), ficha.arquivos.fundo.as_ref())];
    for (remoto, nome) in imagens {
        let (Some(remoto), Some(nome)) = (remoto, nome) else { continue };
        let destino = pasta.join(nome);
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
fn capa_do_acervo(rede: &dyn Rede, modelo: &str, serial: &str, destino: &Path) -> bool {
    if destino.is_file() {
        return true;
    }
    let mut bytes = Vec::new();
    let url = modelo.replace("{serial}", serial);
    if rede.baixar(&url, &mut bytes, LIMITE_IMAGEM, &mut |_| {}).is_err() {
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
