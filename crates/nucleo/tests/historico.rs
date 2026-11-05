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
        format!(
            r#"{{"formato":1,"id":"{id}","jogo":{{"id":"jak2","nome":"Jak II","console":"ps2"}},"versao":"{versao}",
            "estado":"publicada","idioma":"pt-BR","data":"2026-11-05",
            "origem":{{"serial":"SCUS-97265","regiao":"usa","hash_ra":"{HASH}","tamanho":1}},
            "resultado":{{"hash_ra":"{HASH}","tamanho":1}},"tradutores":[],"arquivos":{{"patch":"patch.xdelta"}}}}"#
        ),
    )
    .unwrap();
    let carga = carregar_pacotes(raiz);
    carga.pacotes.into_iter().find(|x| x.ficha.versao == versao).unwrap()
}

fn resultado() -> Resultado {
    Resultado { saida: PathBuf::from("saida.iso"), hash_ra: HASH.into(), compativel_ra: true, segundos: 1.0 }
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
