//! Utilidades só dos testes: ISOs ISO9660 pequenas em memória, servidor HTTP e fichas do catálogo.
#![allow(dead_code)]

pub mod servidor;

pub const HASH_RA: &str = "f51c94dd47cc2eb385d5733389428121";

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Ficha do catálogo com download apontando para `base` (`/<id>/patch`, `/<id>/capa`).
pub fn ficha_catalogo(id: &str, versao: &str, base: &str, hash_ra: &str, tamanho_iso: u64, patch: &[u8], capa: Option<&[u8]>) -> String {
    let capa_dl = capa
        .map(|c| format!(r#","capa":{{"url":"{base}/{id}/capa","sha256":"{}","tamanho":{}}}"#, sha256_hex(c), c.len()))
        .unwrap_or_default();
    format!(
        r#"{{"formato":1,"id":"{id}","jogo":{{"id":"jak2","nome":"Jak II","console":"ps2"}},"versao":"{versao}","estado":"beta",
        "idioma":"pt-BR","data":"2026-11-05",
        "origem":{{"serial":"SCUS-97265","regiao":"usa","hash_ra":"{hash_ra}","tamanho":{tamanho_iso}}},
        "resultado":{{"hash_ra":"{hash_ra}","tamanho":{tamanho_iso}}},"tradutores":[{{"nome":"Fulano"}}],
        "arquivos":{{"patch":"patch.xdelta","capa":"capa.jpg"}},
        "download":{{"patch":{{"url":"{base}/{id}/patch","sha256":"{}","tamanho":{}}}{capa_dl}}}}}"#,
        sha256_hex(patch),
        patch.len()
    )
}

pub fn indice(fichas: &[String]) -> Vec<u8> {
    format!(r#"{{"formato":1,"atualizado":"2026-11-05","traducoes":[{}]}}"#, fichas.join(",")).into_bytes()
}

const SETOR: usize = 2048;

/// Monta uma ISO9660 mínima com os arquivos pedidos. Caminhos podem ter uma
/// subpasta (`DATA/A.BIN`). Os nomes são gravados em maiúsculas com `;1`.
pub fn iso_sintetica(arquivos: &[(&str, &[u8])]) -> Vec<u8> {
    // Separa arquivos da raiz e de cada subpasta, mantendo a ordem.
    let mut pastas: Vec<String> = Vec::new();
    for (caminho, _) in arquivos {
        if let Some((pasta, _)) = caminho.split_once('/') {
            let pasta = pasta.to_uppercase();
            if !pastas.contains(&pasta) {
                pastas.push(pasta);
            }
        }
    }

    // Setores: 16 = PVD, 17 = terminador, 18 = raiz, 19.. = subpastas, depois os dados.
    let setor_raiz = 18usize;
    let setor_pasta = |i: usize| 19 + i;
    let mut proximo = 19 + pastas.len();
    let mut posicoes = Vec::new();
    for (_, dados) in arquivos {
        posicoes.push(proximo);
        proximo += dados.len().div_ceil(SETOR).max(1);
    }
    let total = proximo;
    let mut iso = vec![0u8; total * SETOR];

    // Arquivos.
    for ((_, dados), setor) in arquivos.iter().zip(&posicoes) {
        iso[setor * SETOR..setor * SETOR + dados.len()].copy_from_slice(dados);
    }

    // Diretórios.
    let mut raiz = Vec::new();
    raiz.extend(registro(setor_raiz, SETOR, true, &[0]));
    raiz.extend(registro(setor_raiz, SETOR, true, &[1]));
    for (i, pasta) in pastas.iter().enumerate() {
        raiz.extend(registro(setor_pasta(i), SETOR, true, pasta.as_bytes()));
    }
    for ((caminho, dados), setor) in arquivos.iter().zip(&posicoes) {
        if !caminho.contains('/') {
            let nome = format!("{};1", caminho.to_uppercase());
            raiz.extend(registro(*setor, dados.len(), false, nome.as_bytes()));
        }
    }
    iso[setor_raiz * SETOR..setor_raiz * SETOR + raiz.len()].copy_from_slice(&raiz);

    for (i, pasta) in pastas.iter().enumerate() {
        let mut dir = Vec::new();
        dir.extend(registro(setor_pasta(i), SETOR, true, &[0]));
        dir.extend(registro(setor_raiz, SETOR, true, &[1]));
        for ((caminho, dados), setor) in arquivos.iter().zip(&posicoes) {
            if let Some((p, nome)) = caminho.split_once('/') {
                if p.to_uppercase() == *pasta {
                    let nome = format!("{};1", nome.to_uppercase());
                    dir.extend(registro(*setor, dados.len(), false, nome.as_bytes()));
                }
            }
        }
        let s = setor_pasta(i) * SETOR;
        iso[s..s + dir.len()].copy_from_slice(&dir);
    }

    // Descritor primário (PVD).
    let pvd = &mut iso[16 * SETOR..17 * SETOR];
    pvd[0] = 1;
    pvd[1..6].copy_from_slice(b"CD001");
    pvd[6] = 1;
    ambos32(&mut pvd[80..88], total as u32);
    ambos16(&mut pvd[128..132], SETOR as u16);
    let r = registro(setor_raiz, SETOR, true, &[0]);
    pvd[156..156 + r.len()].copy_from_slice(&r);

    // Terminador.
    let fim = &mut iso[17 * SETOR..18 * SETOR];
    fim[0] = 255;
    fim[1..6].copy_from_slice(b"CD001");
    fim[6] = 1;
    iso
}

fn registro(setor: usize, tamanho: usize, pasta: bool, nome: &[u8]) -> Vec<u8> {
    let mut len = 33 + nome.len();
    if len % 2 == 1 {
        len += 1;
    }
    let mut r = vec![0u8; len];
    r[0] = len as u8;
    ambos32(&mut r[2..10], setor as u32);
    ambos32(&mut r[10..18], tamanho as u32);
    r[25] = if pasta { 2 } else { 0 };
    ambos16(&mut r[28..32], 1);
    r[32] = nome.len() as u8;
    r[33..33 + nome.len()].copy_from_slice(nome);
    r
}

fn ambos32(dest: &mut [u8], v: u32) {
    dest[..4].copy_from_slice(&v.to_le_bytes());
    dest[4..8].copy_from_slice(&v.to_be_bytes());
}

fn ambos16(dest: &mut [u8], v: u16) {
    dest[..2].copy_from_slice(&v.to_le_bytes());
    dest[2..4].copy_from_slice(&v.to_be_bytes());
}

/// Grava os bytes num arquivo dentro de `pasta` e devolve o caminho.
pub fn gravar(pasta: &std::path::Path, nome: &str, bytes: &[u8]) -> std::path::PathBuf {
    let caminho = pasta.join(nome);
    std::fs::write(&caminho, bytes).unwrap();
    caminho
}
