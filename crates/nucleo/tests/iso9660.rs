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

#[test]
fn iso_dentro_de_bin_mode2() {
    use nucleo::Setores;
    let pasta = tempfile::tempdir().unwrap();
    let bin = comum::bin_de_iso(&comum::iso_sintetica(&[("SYSTEM.CNF", CNF), ("DATA/A.BIN", b"conteudo")]), 24);
    let caminho = comum::gravar(pasta.path(), "jogo.bin", &bin);
    let mut iso = Iso::abrir_com(&caminho, Setores::Bin { tamanho: 2352, dados: 24 }).unwrap();
    assert_eq!(iso.ler_arquivo("SYSTEM.CNF").unwrap(), CNF);
    assert_eq!(iso.ler_arquivo("DATA/A.BIN").unwrap(), b"conteudo");
    assert_eq!(iso.tamanho(), bin.len() as u64);
}

#[test]
fn iso_dentro_de_bin_mode1() {
    use nucleo::{Setores, TipoFaixa};
    let pasta = tempfile::tempdir().unwrap();
    let bin = comum::bin_de_iso(&comum::iso_sintetica(&[("SYSTEM.CNF", CNF)]), 16);
    let caminho = comum::gravar(pasta.path(), "jogo.bin", &bin);
    let mut iso = Iso::abrir_com(&caminho, Setores::do_tipo(TipoFaixa::Mode1_2352)).unwrap();
    assert_eq!(iso.ler_arquivo("SYSTEM.CNF").unwrap(), CNF);
}

#[test]
fn arquivo_que_atravessa_setores_no_bin() {
    use nucleo::Setores;
    let pasta = tempfile::tempdir().unwrap();
    let grande: Vec<u8> = (0..5000u32).map(|i| (i % 251) as u8).collect();
    let bin = comum::bin_de_iso(&comum::iso_sintetica(&[("SYSTEM.CNF", CNF), ("G.BIN", &grande)]), 24);
    let caminho = comum::gravar(pasta.path(), "jogo.bin", &bin);
    let mut iso = Iso::abrir_com(&caminho, Setores::Bin { tamanho: 2352, dados: 24 }).unwrap();
    assert_eq!(iso.ler_arquivo("G.BIN").unwrap(), grande);
}
