mod comum;

use std::fs;

use md5::{Digest, Md5};
use nucleo::{Erro, Regiao, TipoFaixa, hash_ra_ps1, identificar_bin_ps1, identificar_ps1};

fn md5_hex(bytes: &[u8]) -> String {
    Md5::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Executável `PS-X EXE` com `t_size` no cabeçalho e `corpo` bytes depois dos 2048 do cabeçalho.
fn exe(t_size: u32, corpo: usize) -> Vec<u8> {
    let mut b = vec![0u8; 2048 + corpo];
    b[..8].copy_from_slice(b"PS-X EXE");
    b[28..32].copy_from_slice(&t_size.to_le_bytes());
    for (i, x) in b[2048..].iter_mut().enumerate() {
        *x = (i % 253) as u8 + 1;
    }
    b
}

/// `.cue` + `.bin` (MODE2/2352) com os arquivos dados; devolve a pasta e o caminho do `.cue`.
fn disco(arquivos: &[(&str, &[u8])]) -> (tempfile::TempDir, std::path::PathBuf) {
    let pasta = tempfile::tempdir().unwrap();
    let bin = comum::bin_de_iso(&comum::iso_sintetica(arquivos), 24);
    comum::gravar(pasta.path(), "Jogo (USA).bin", &bin);
    let cue = comum::gravar(pasta.path(), "Jogo (USA).cue", b"FILE \"Jogo (USA).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
    (pasta, cue)
}

const CNF: &[u8] = b"BOOT = cdrom:\\SLUS_006.84;1\r\nTCB = 4\r\nEVENT = 10\r\nSTACK = 801FFFF0\r\n";

#[test]
fn identifica_serial_pelo_boot() {
    let e = exe(4096, 4096);
    let (pasta, cue) = disco(&[("SYSTEM.CNF", CNF), ("SLUS_006.84", &e)]);
    let id = identificar_ps1(&cue).unwrap();
    assert_eq!(id.serial, "SLUS-00684");
    assert_eq!(id.regiao, Regiao::Usa);
    assert_eq!(id.executavel, "SLUS_006.84");
    let bin = fs::metadata(pasta.path().join("Jogo (USA).bin")).unwrap().len();
    assert_eq!(id.tamanho_dados, bin);
    assert_eq!(id.tamanho_total, bin);
    let mut esperado = b"SLUS_006.84".to_vec();
    esperado.extend(&e[..4096 + 2048]);
    assert_eq!(id.hash_ra, md5_hex(&esperado));
}

#[test]
fn sem_system_cnf_usa_psx_exe() {
    let e = exe(2048, 2048);
    let (_p, cue) = disco(&[("PSX.EXE", &e)]);
    let id = identificar_ps1(&cue).unwrap();
    assert_eq!(id.executavel, "PSX.EXE");
    let mut esperado = b"PSX.EXE".to_vec();
    esperado.extend(&e);
    assert_eq!(id.hash_ra, md5_hex(&esperado));
}

#[test]
fn hash_usa_so_o_tamanho_do_cabecalho() {
    let mut e = exe(4096, 4096);
    e.extend(vec![0x55u8; 3000]); // lixo depois do que o cabeçalho declara
    let mut esperado = b"SLUS_006.84".to_vec();
    esperado.extend(&e[..4096 + 2048]);
    assert_eq!(hash_ra_ps1("SLUS_006.84", &e), md5_hex(&esperado));
}

#[test]
fn sem_cabecalho_ps_x_usa_o_arquivo_inteiro() {
    assert_eq!(hash_ra_ps1("A.EXE", b"abc"), md5_hex(b"A.EXEabc"));
}

#[test]
fn executavel_em_subpasta_entra_no_hash_com_a_pasta() {
    // Como no rcheevos: o nome no hash é o caminho do BOOT, sem "cdrom:" e sem as barras do início.
    let e = exe(2048, 2048);
    let (_p, cue) = disco(&[("SYSTEM.CNF", b"BOOT = cdrom:\\GAME\\SLES_123.45;1\r\n"), ("GAME/SLES_123.45", &e)]);
    let id = identificar_ps1(&cue).unwrap();
    assert_eq!(id.executavel, "GAME\\SLES_123.45");
    assert_eq!(id.serial, "SLES-12345");
    assert_eq!(id.regiao, Regiao::Europa);
    let mut esperado = b"GAME\\SLES_123.45".to_vec();
    esperado.extend(&e);
    assert_eq!(id.hash_ra, md5_hex(&esperado));
}

#[test]
fn identificar_bin_direto() {
    let e = exe(2048, 2048);
    let pasta = tempfile::tempdir().unwrap();
    let bin = comum::bin_de_iso(&comum::iso_sintetica(&[("SYSTEM.CNF", CNF), ("SLUS_006.84", &e)]), 24);
    let caminho = comum::gravar(pasta.path(), "x.bin.parcial", &bin);
    let id = identificar_bin_ps1(&caminho, TipoFaixa::Mode2_2352).unwrap();
    assert_eq!(id.serial, "SLUS-00684");
    assert_eq!(id.tamanho_dados, bin.len() as u64);
}

#[test]
fn soma_o_tamanho_das_faixas_de_audio() {
    let e = exe(2048, 2048);
    let pasta = tempfile::tempdir().unwrap();
    let bin = comum::bin_de_iso(&comum::iso_sintetica(&[("SYSTEM.CNF", CNF), ("SLUS_006.84", &e)]), 24);
    comum::gravar(pasta.path(), "J (Track 1).bin", &bin);
    comum::gravar(pasta.path(), "J (Track 2).bin", &vec![0u8; 2352 * 10]);
    let cue = comum::gravar(
        pasta.path(),
        "J.cue",
        b"FILE \"J (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"J (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 01 00:00:00\n",
    );
    let id = identificar_ps1(&cue).unwrap();
    assert_eq!(id.tamanho_dados, bin.len() as u64);
    assert_eq!(id.tamanho_total, bin.len() as u64 + 2352 * 10);
}

#[test]
fn nao_e_ps1() {
    let (_p, cue) = disco(&[("OUTRO.TXT", b"oi")]);
    let erro = identificar_ps1(&cue).unwrap_err();
    assert!(matches!(erro, Erro::NaoEPs1));
    assert_eq!(erro.mensagem(), "Este arquivo não parece ser de um jogo de PlayStation.");
}

#[test]
fn bin_que_nao_e_cd_nao_e_ps1() {
    let pasta = tempfile::tempdir().unwrap();
    comum::gravar(pasta.path(), "a.bin", &vec![0u8; 2352 * 40]);
    let cue = comum::gravar(pasta.path(), "a.cue", b"FILE \"a.bin\" BINARY\n  TRACK 01 MODE2/2352\n");
    assert!(matches!(identificar_ps1(&cue), Err(Erro::NaoEPs1)));
}

#[test]
#[ignore = "precisa de um jogo de PS1 real em TRADUMIX_CUE_PS1 e do hash em TRADUMIX_HASH_RA_PS1"]
fn jogo_real_ps1() {
    let cue = std::env::var("TRADUMIX_CUE_PS1").expect("defina TRADUMIX_CUE_PS1");
    let hash = std::env::var("TRADUMIX_HASH_RA_PS1").expect("defina TRADUMIX_HASH_RA_PS1");
    assert_eq!(identificar_ps1(std::path::Path::new(&cue)).unwrap().hash_ra, hash.to_lowercase());
}

#[test]
fn t_size_maior_que_o_arquivo_le_os_setores_seguintes() {
    // O rcheevos lê t_size + 2048 bytes a partir do primeiro setor do executável, mesmo que o
    // arquivo seja menor; o que vem depois na imagem entra no hash.
    let e = exe(4096, 2048); // declara 4096 + 2048, tem só 2048 + 2048
    let depois = vec![0x77u8; 2048];
    let (_p, cue) = disco(&[("SYSTEM.CNF", CNF), ("SLUS_006.84", &e), ("Z.DAT", &depois)]);
    let mut esperado = b"SLUS_006.84".to_vec();
    esperado.extend(&e);
    esperado.extend(&depois);
    assert_eq!(identificar_ps1(&cue).unwrap().hash_ra, md5_hex(&esperado));
}
