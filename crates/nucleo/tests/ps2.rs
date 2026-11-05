mod comum;

use md5::{Digest, Md5};
use nucleo::{Erro, Regiao, hash_ra, identificar, regiao_do_serial, serial_do_executavel};

fn md5_hex(bytes: &[u8]) -> String {
    Md5::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn iso(arquivos: &[(&str, &[u8])]) -> (tempfile::TempDir, std::path::PathBuf) {
    let pasta = tempfile::tempdir().unwrap();
    let caminho = comum::gravar(pasta.path(), "jogo.iso", &comum::iso_sintetica(arquivos));
    (pasta, caminho)
}

#[test]
fn hash_do_nome_mais_bytes() {
    assert_eq!(hash_ra("SCUS_972.65", b"abc"), md5_hex(b"SCUS_972.65abc"));
}

#[test]
fn serial_e_regiao() {
    assert_eq!(serial_do_executavel("SCUS_972.65"), "SCUS-97265");
    assert_eq!(regiao_do_serial("SCUS-97265"), Regiao::Usa);
    assert_eq!(regiao_do_serial("SLES-12345"), Regiao::Europa);
    assert_eq!(regiao_do_serial("SLPM-12345"), Regiao::Japao);
    assert_eq!(regiao_do_serial("XXXX-00000"), Regiao::Desconhecida);
    assert_eq!(Regiao::Usa.nome(), "USA");
    assert_eq!(Regiao::Japao.nome(), "Japão");
}

#[test]
fn identifica_iso_sintetica() {
    let (_p, caminho) = iso(&[
        ("SYSTEM.CNF", b"BOOT2 = cdrom0:\\SCUS_972.65;1\r\nVER = 1.00\r\n"),
        ("SCUS_972.65", b"ELF"),
    ]);
    let id = identificar(&caminho).unwrap();
    assert_eq!(id.executavel, "SCUS_972.65");
    assert_eq!(id.serial, "SCUS-97265");
    assert_eq!(id.regiao, Regiao::Usa);
    assert_eq!(id.hash_ra, hash_ra("SCUS_972.65", b"ELF"));
    assert_eq!(id.tamanho, std::fs::metadata(&caminho).unwrap().len());
}

#[test]
fn boot2_com_espacos_e_minusculas() {
    let (_p, caminho) = iso(&[
        ("SYSTEM.CNF", b"boot2=cdrom0:\\scus_972.65;1\n"),
        ("SCUS_972.65", b"ELF"),
    ]);
    let id = identificar(&caminho).unwrap();
    assert_eq!(id.serial, "SCUS-97265");
    assert_eq!(id.hash_ra, hash_ra("SCUS_972.65", b"ELF"));
}

#[test]
fn iso_sem_system_cnf_nao_e_ps2() {
    let (_p, caminho) = iso(&[("OUTRO.BIN", b"x")]);
    assert!(matches!(identificar(&caminho), Err(Erro::NaoEPs2)));
}

#[test]
fn system_cnf_sem_boot2_nao_e_ps2() {
    let (_p, caminho) = iso(&[("SYSTEM.CNF", b"VER = 1.00\r\n")]);
    assert!(matches!(identificar(&caminho), Err(Erro::NaoEPs2)));
}

#[test]
#[ignore = "precisa da ISO original do Jak II em TRADUMIX_ISO_JAK2"]
fn jak2_real() {
    let caminho = std::env::var("TRADUMIX_ISO_JAK2").expect("defina TRADUMIX_ISO_JAK2");
    let id = identificar(std::path::Path::new(&caminho)).unwrap();
    assert_eq!(id.serial, "SCUS-97265");
    assert_eq!(id.hash_ra, "f51c94dd47cc2eb385d5733389428121");
    assert_eq!(id.tamanho, 4_410_671_104);
}
