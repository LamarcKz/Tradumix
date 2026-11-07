mod comum;

use std::fs;
use std::path::PathBuf;

use nucleo::{Historico, Pacote, Resultado, carregar_pacotes};

const HASH: &str = "f51c94dd47cc2eb385d5733389428121";

fn pacote(raiz: &std::path::Path, id: &str, versao: &str) -> Pacote {
    let p = raiz.join(format!("{id}-{versao}"));
    fs::create_dir_all(&p).unwrap();
    fs::write(p.join("patch.xdelta"), b"VCD").unwrap();
    fs::write(
        p.join("ficha.json"),
        comum::ficha_ps2(id, versao, HASH, 1, 1, "https://x/p", "3ce44ebfecc30fc110c57f4fdf6ec0cd8c483fb3403d60ef3a671afb5f26e8de", 3, ""),
    )
    .unwrap();
    let carga = carregar_pacotes(raiz);
    carga.pacotes.into_iter().find(|x| x.ficha.versao == versao).unwrap()
}

fn resultado() -> Resultado {
    Resultado { saida: PathBuf::from("saida.iso"), hash_ra: Some(HASH.into()), compativel_ra: Some(true), segundos: 1.0 }
}

#[test]
fn salva_e_carrega() {
    let pasta = tempfile::tempdir().unwrap();
    let p = pacote(pasta.path(), "jak2-usa-ptbr", "1.0");
    let arquivo = pasta.path().join("historico.json");
    let mut h = Historico::carregar(&arquivo);
    assert!(h.entradas.is_empty());
    h.registrar(&p, &resultado(), "2026-11-05");
    h.salvar(&arquivo).unwrap();
    let lido = Historico::carregar(&arquivo);
    assert_eq!(lido.entradas.len(), 1);
    assert_eq!(lido.entradas[0].pacote_id, "jak2-usa-ptbr");
    assert_eq!(lido.entradas[0].versao, "1.0");
    assert_eq!(lido.entradas[0].saida, PathBuf::from("saida.iso"));
    assert_eq!(lido.entradas[0].data, "2026-11-05");
}

#[test]
fn arquivo_corrompido_vira_vazio() {
    let pasta = tempfile::tempdir().unwrap();
    let arquivo = pasta.path().join("historico.json");
    fs::write(&arquivo, "{ quebrado").unwrap();
    assert!(Historico::carregar(&arquivo).entradas.is_empty());
}

#[test]
fn aponta_atualizacao_quando_ha_versao_maior() {
    let pasta = tempfile::tempdir().unwrap();
    let v10 = pacote(&pasta.path().join("a"), "jak2-usa-ptbr", "1.0");
    let v11 = pacote(&pasta.path().join("b"), "jak2-usa-ptbr", "1.1");
    let mut h = Historico::default();
    h.registrar(&v10, &resultado(), "2026-11-05");
    let atualizacoes = h.atualizacoes(&[v11]);
    assert_eq!(atualizacoes.len(), 1);
    assert_eq!(atualizacoes[0].pacote_id, "jak2-usa-ptbr");
    assert_eq!(atualizacoes[0].versao_gerada, "1.0");
    assert_eq!(atualizacoes[0].versao_nova, "1.1");
}

#[test]
fn sem_atualizacao_na_mesma_versao() {
    let pasta = tempfile::tempdir().unwrap();
    let v10 = pacote(pasta.path(), "jak2-usa-ptbr", "1.0");
    let mut h = Historico::default();
    h.registrar(&v10, &resultado(), "2026-11-05");
    assert!(h.atualizacoes(&[v10]).is_empty());
}

#[test]
fn considera_so_a_versao_mais_nova_gerada() {
    let pasta = tempfile::tempdir().unwrap();
    let v10 = pacote(&pasta.path().join("a"), "jak2-usa-ptbr", "1.0");
    let v11 = pacote(&pasta.path().join("b"), "jak2-usa-ptbr", "1.1");
    let mut h = Historico::default();
    h.registrar(&v10, &resultado(), "2026-11-05");
    h.registrar(&v11, &resultado(), "2026-11-06");
    assert!(h.atualizacoes(&[v11]).is_empty());
}
