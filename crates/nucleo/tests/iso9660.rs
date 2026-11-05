mod comum;

use nucleo::{Erro, Iso};

const CNF: &[u8] = b"BOOT2 = cdrom0:\\SCUS_972.65;1\r\n";

fn abrir(arquivos: &[(&str, &[u8])]) -> (tempfile::TempDir, Iso) {
    let pasta = tempfile::tempdir().unwrap();
    let caminho = comum::gravar(pasta.path(), "teste.iso", &comum::iso_sintetica(arquivos));
    let iso = Iso::abrir(&caminho).unwrap();
    (pasta, iso)
}

#[test]
fn le_arquivo_da_raiz() {
    let (_p, mut iso) = abrir(&[("SYSTEM.CNF", CNF)]);
    assert_eq!(iso.ler_arquivo("system.cnf").unwrap(), CNF);
}

#[test]
fn le_arquivo_em_subpasta() {
    let (_p, mut iso) = abrir(&[("SYSTEM.CNF", CNF), ("DATA/A.BIN", b"conteudo")]);
    assert_eq!(iso.ler_arquivo("DATA/A.BIN").unwrap(), b"conteudo");
}

#[test]
fn aceita_sufixo_de_versao() {
    let (_p, mut iso) = abrir(&[("SYSTEM.CNF", CNF)]);
    assert_eq!(iso.ler_arquivo("SYSTEM.CNF;1").unwrap(), CNF);
}

#[test]
fn arquivo_ausente_da_erro() {
    let (_p, mut iso) = abrir(&[("SYSTEM.CNF", CNF)]);
    assert!(matches!(iso.ler_arquivo("NADA.BIN"), Err(Erro::ArquivoNaoEncontrado(_))));
}

#[test]
fn arquivo_que_nao_e_iso_da_erro() {
    let pasta = tempfile::tempdir().unwrap();
    let caminho = comum::gravar(pasta.path(), "zeros.iso", &vec![0u8; 2 * 1024 * 1024]);
    assert!(matches!(Iso::abrir(&caminho), Err(Erro::IsoInvalida)));
}

#[test]
fn informa_o_tamanho_do_arquivo() {
    let (_p, iso) = abrir(&[("SYSTEM.CNF", CNF)]);
    assert_eq!(iso.tamanho(), 20 * 2048);
}
