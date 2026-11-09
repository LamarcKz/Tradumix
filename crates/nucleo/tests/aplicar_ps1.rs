//! Aplicar tradução de PS1: `.cue` + `.bin` de dados (com o patch) + faixas de áudio (copiadas).
mod comum;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;

use nucleo::{Erro, Pacote, Pedido, aplicar, carregar_pacotes, conferir, identificar_ps1, nome_saida};

const CNF: &[u8] = b"BOOT = cdrom:\\SLUS_006.84;1\r\nTCB = 4\r\n";

fn xdelta3() -> PathBuf {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/binaries/xdelta3-x86_64-pc-windows-msvc.exe");
    assert!(p.is_file(), "xdelta3 não encontrado: rode npm run xdelta3");
    p
}

fn dados(semente: u8, tamanho: usize) -> Vec<u8> {
    (0..tamanho).map(|i| (i as u8).wrapping_mul(29).wrapping_add(semente)).collect()
}

fn bin_do_jogo(semente_dados: u8) -> Vec<u8> {
    let exe = comum::exe_psx(4096, 3);
    comum::bin_de_iso(&comum::iso_sintetica(&[("SYSTEM.CNF", CNF), ("SLUS_006.84", &exe), ("DATA.BIN", &dados(semente_dados, 60_000))]), 24)
}

struct Cenario {
    _pasta: tempfile::TempDir,
    raiz: PathBuf,
    cue: PathBuf,
    destino: Vec<u8>,
    audios: Vec<Vec<u8>>,
    pacote: Pacote,
}

/// Disco de PS1 com `faixas_audio` faixas de áudio; o patch troca bytes do DATA.BIN (fora do executável).
fn cenario(faixas_audio: usize) -> Cenario {
    let pasta = tempfile::tempdir().unwrap();
    let raiz = pasta.path().join("Jogo PS1");
    fs::create_dir_all(&raiz).unwrap();
    let origem = bin_do_jogo(1);
    let destino = bin_do_jogo(2);
    let mut cue = String::new();
    let nome_dados = if faixas_audio == 0 { "Jogo (USA).bin".to_string() } else { "Jogo (USA) (Track 1).bin".to_string() };
    comum::gravar(&raiz, &nome_dados, &origem);
    cue.push_str(&format!("FILE \"{nome_dados}\" BINARY\r\n  TRACK 01 MODE2/2352\r\n    INDEX 01 00:00:00\r\n"));
    let mut audios = Vec::new();
    for n in 0..faixas_audio {
        let faixa = n + 2;
        let audio = dados(40 + n as u8, 2352 * 15);
        let nome = format!("Jogo (USA) (Track {faixa}).bin");
        comum::gravar(&raiz, &nome, &audio);
        cue.push_str(&format!("FILE \"{nome}\" BINARY\r\n  TRACK {faixa:02} AUDIO\r\n    INDEX 00 00:00:00\r\n    INDEX 01 00:02:00\r\n"));
        audios.push(audio);
    }
    let caminho_cue = comum::gravar(&raiz, "Jogo (USA).cue", cue.as_bytes());

    let destino_tmp = comum::gravar(pasta.path(), "destino.tmp", &destino);
    let dir_pacote = pasta.path().join("pacotes").join("jogo-ps1");
    fs::create_dir_all(&dir_pacote).unwrap();
    let ok = Command::new(xdelta3())
        .args(["-e", "-9", "-S", "djw", "-f", "-s"])
        .arg(raiz.join(&nome_dados))
        .arg(&destino_tmp)
        .arg(dir_pacote.join("patch.xdelta"))
        .env_remove("XDELTA")
        .status()
        .unwrap();
    assert!(ok.success());

    let id = identificar_ps1(&caminho_cue).unwrap();
    let patch = fs::read(dir_pacote.join("patch.xdelta")).unwrap();
    let ficha = comum::ficha_ps1(
        "jogo-ps1-usa-ptbr",
        &id.hash_ra,
        &id.hash_ra,
        origem.len() as u64,
        destino.len() as u64,
        &comum::sha256_hex(&patch),
        patch.len() as u64,
        "",
    );
    fs::write(dir_pacote.join("ficha.json"), ficha).unwrap();
    let carga = carregar_pacotes(&pasta.path().join("pacotes"));
    assert!(carga.invalidos.is_empty(), "{:?}", carga.invalidos);
    let pacote = carga.pacotes.into_iter().next().unwrap();
    Cenario { _pasta: pasta, raiz, cue: caminho_cue, destino, audios, pacote }
}

fn pedido(c: &Cenario, saida: PathBuf) -> Pedido {
    Pedido { iso_original: c.cue.clone(), saida, sobrescrever: false, xdelta3: xdelta3() }
}

fn arquivos(pasta: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(pasta).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    v.sort();
    v
}

#[test]
fn nome_de_saida_ps1() {
    let c = cenario(0);
    assert_eq!(nome_saida(&c.pacote, &c.cue), c.raiz.join("Jogo de PS1 (USA) PT-BR v1.0.cue"));
}

#[test]
fn aplicar_ps1_uma_faixa() {
    let c = cenario(0);
    let saida = nome_saida(&c.pacote, &c.cue);
    let r = aplicar(&c.pacote, &pedido(&c, saida.clone()), &mut |_, _| {}, &AtomicBool::new(false)).unwrap();
    assert_eq!(r.saida, saida);
    assert_eq!(r.compativel_ra, Some(true));
    let bin = c.raiz.join("Jogo de PS1 (USA) PT-BR v1.0.bin");
    assert_eq!(fs::read(&bin).unwrap(), c.destino);
    let cue = fs::read_to_string(&saida).unwrap();
    assert_eq!(cue, "FILE \"Jogo de PS1 (USA) PT-BR v1.0.bin\" BINARY\r\n  TRACK 01 MODE2/2352\r\n    INDEX 01 00:00:00\r\n");
    // O .cue novo abre e aponta para o .bin novo.
    assert_eq!(identificar_ps1(&saida).unwrap().hash_ra, r.hash_ra.unwrap());
    assert_eq!(
        arquivos(&c.raiz),
        ["Jogo (USA).bin", "Jogo (USA).cue", "Jogo de PS1 (USA) PT-BR v1.0.bin", "Jogo de PS1 (USA) PT-BR v1.0.cue"]
    );
}

#[test]
fn aplicar_ps1_com_audio() {
    let c = cenario(2);
    let saida = nome_saida(&c.pacote, &c.cue);
    aplicar(&c.pacote, &pedido(&c, saida.clone()), &mut |_, _| {}, &AtomicBool::new(false)).unwrap();
    let base = "Jogo de PS1 (USA) PT-BR v1.0";
    assert_eq!(fs::read(c.raiz.join(format!("{base} (Faixa 1).bin"))).unwrap(), c.destino);
    assert_eq!(fs::read(c.raiz.join(format!("{base} (Faixa 2).bin"))).unwrap(), c.audios[0]);
    assert_eq!(fs::read(c.raiz.join(format!("{base} (Faixa 3).bin"))).unwrap(), c.audios[1]);
    let cue = fs::read_to_string(&saida).unwrap();
    let files: Vec<&str> = cue.lines().filter(|l| l.starts_with("FILE")).collect();
    assert_eq!(
        files,
        [
            format!("FILE \"{base} (Faixa 1).bin\" BINARY"),
            format!("FILE \"{base} (Faixa 2).bin\" BINARY"),
            format!("FILE \"{base} (Faixa 3).bin\" BINARY")
        ]
    );
    assert!(cue.contains("TRACK 03 AUDIO\r\n    INDEX 00 00:00:00\r\n    INDEX 01 00:02:00"));
}

#[test]
fn aplicar_ps1_falha_nao_deixa_lixo() {
    let c = cenario(2);
    let patch = c.pacote.patch();
    let bytes = fs::read(&patch).unwrap();
    fs::write(&patch, &bytes[..bytes.len() / 2]).unwrap();
    let antes = arquivos(&c.raiz);
    let r = aplicar(&c.pacote, &pedido(&c, nome_saida(&c.pacote, &c.cue)), &mut |_, _| {}, &AtomicBool::new(false));
    assert!(matches!(r, Err(Erro::XdeltaFalhou(_))), "{r:?}");
    assert_eq!(arquivos(&c.raiz), antes);
}

#[test]
fn aplicar_ps1_recusa_se_uma_faixa_ja_existe() {
    let c = cenario(2);
    let saida = nome_saida(&c.pacote, &c.cue);
    let faixa = comum::gravar(&c.raiz, "Jogo de PS1 (USA) PT-BR v1.0 (Faixa 3).bin", b"antigo");
    let r = aplicar(&c.pacote, &pedido(&c, saida.clone()), &mut |_, _| {}, &AtomicBool::new(false));
    assert!(matches!(&r, Err(Erro::SaidaJaExiste(p)) if *p == faixa), "{r:?}");
    assert_eq!(fs::read(&faixa).unwrap(), b"antigo");
    assert!(!saida.exists());
}

#[test]
fn conferir_ps1_jogo_errado() {
    let c = cenario(0);
    let exe = comum::exe_psx(2048, 9);
    let outro = comum::bin_de_iso(&comum::iso_sintetica(&[("SYSTEM.CNF", b"BOOT = cdrom:\\SLES_123.45;1\r\n"), ("SLES_123.45", &exe)]), 24);
    comum::gravar(&c.raiz, "outro.bin", &outro);
    let cue = comum::gravar(&c.raiz, "outro.cue", b"FILE \"outro.bin\" BINARY\n  TRACK 01 MODE2/2352\n");
    match conferir(&c.pacote, &cue) {
        Err(Erro::JogoErrado { esperado_serial, encontrado_serial, .. }) => {
            assert_eq!(esperado_serial, "SLUS-00684");
            assert_eq!(encontrado_serial, "SLES-12345");
        }
        outro => panic!("esperava JogoErrado, veio {outro:?}"),
    }
}

#[test]
fn conferir_ps1_tamanho_diferente() {
    let c = cenario(0);
    let mut maior = bin_do_jogo(1);
    maior.extend(vec![0u8; 2352]);
    comum::gravar(&c.raiz, "maior.bin", &maior);
    let cue = comum::gravar(&c.raiz, "maior.cue", b"FILE \"maior.bin\" BINARY\n  TRACK 01 MODE2/2352\n");
    assert!(matches!(conferir(&c.pacote, &cue), Err(Erro::TamanhoDiferente { .. })));
}

#[test]
fn conferir_ps1_arquivo_que_nao_e_cue() {
    let c = cenario(0);
    let falso = comum::gravar(&c.raiz, "nada.cue", b"isto nao e um cue");
    assert!(matches!(conferir(&c.pacote, &falso), Err(Erro::CueInvalido(_))));
}

#[test]
fn falha_ao_copiar_audio_desfaz_o_que_foi_criado() {
    let c = cenario(2);
    let saida = nome_saida(&c.pacote, &c.cue);
    // Uma pasta no lugar da faixa 3 faz a cópia do áudio falhar depois do patch já gravado.
    fs::create_dir(c.raiz.join("Jogo de PS1 (USA) PT-BR v1.0 (Faixa 3).bin")).unwrap();
    let mut p = pedido(&c, saida.clone());
    p.sobrescrever = true;
    let r = aplicar(&c.pacote, &p, &mut |_, _| {}, &AtomicBool::new(false));
    assert!(matches!(r, Err(Erro::Io(_))), "{r:?}");
    assert!(!saida.exists());
    assert!(!c.raiz.join("Jogo de PS1 (USA) PT-BR v1.0 (Faixa 1).bin").exists());
    assert!(!c.raiz.join("Jogo de PS1 (USA) PT-BR v1.0 (Faixa 2).bin").exists());
}
