mod comum;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;

use nucleo::{Erro, Etapa, Pacote, Pedido, aplicar, carregar_pacotes, conferir, identificar, nome_saida};

const CNF: &[u8] = b"BOOT2 = cdrom0:\\SCUS_972.65;1\r\n";

fn xdelta3() -> PathBuf {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/binaries/xdelta3-x86_64-pc-windows-msvc.exe");
    assert!(p.is_file(), "xdelta3 não encontrado: rode npm run xdelta3");
    p
}

fn dados(semente: u8, tamanho: usize) -> Vec<u8> {
    (0..tamanho).map(|i| (i as u8).wrapping_mul(31).wrapping_add(semente)).collect()
}

struct Cenario {
    _pasta: tempfile::TempDir,
    raiz: PathBuf,
    original: PathBuf,
    destino: Vec<u8>,
    pacote: Pacote,
}

/// Monta origem e destino (mesmo executável, dados diferentes), gera o patch com o
/// xdelta3 e grava o pacote. `nome_pasta` permite testar caminhos com acento.
fn cenario(nome_pasta: &str) -> Cenario {
    let pasta = tempfile::tempdir().unwrap();
    let raiz = pasta.path().join(nome_pasta);
    fs::create_dir_all(&raiz).unwrap();
    let exe = dados(7, 5000);
    let origem = comum::iso_sintetica(&[("SYSTEM.CNF", CNF), ("SCUS_972.65", &exe), ("DATA/A.BIN", &dados(1, 300_000))]);
    let destino = comum::iso_sintetica(&[("SYSTEM.CNF", CNF), ("SCUS_972.65", &exe), ("DATA/A.BIN", &dados(2, 300_000))]);
    let original = comum::gravar(&raiz, "Original (USA).iso", &origem);
    let caminho_destino = comum::gravar(&raiz, "destino.tmp", &destino);

    let dir_pacote = raiz.join("pacote").join("jak2");
    fs::create_dir_all(&dir_pacote).unwrap();
    let ok = Command::new(xdelta3())
        .args(["-e", "-9", "-S", "djw", "-f", "-s"])
        .arg(&original)
        .arg(&caminho_destino)
        .arg(dir_pacote.join("patch.xdelta"))
        .env_remove("XDELTA")
        .status()
        .unwrap();
    assert!(ok.success());
    fs::remove_file(&caminho_destino).unwrap();

    let id = identificar(&original).unwrap();
    let ficha = format!(
        r#"{{"formato":1,"id":"jak2-usa-ptbr","jogo":{{"id":"jak2","nome":"Jak II","console":"ps2"}},
        "versao":"1.0","estado":"rascunho","idioma":"pt-BR","data":"2026-11-05",
        "origem":{{"serial":"SCUS-97265","regiao":"usa","hash_ra":"{h}","tamanho":{to}}},
        "resultado":{{"hash_ra":"{h}","tamanho":{td}}},
        "tradutores":[{{"nome":"Fulano"}}],"arquivos":{{"patch":"patch.xdelta"}}}}"#,
        h = id.hash_ra,
        to = origem.len(),
        td = destino.len()
    );
    fs::write(dir_pacote.join("ficha.json"), ficha).unwrap();
    let carga = carregar_pacotes(&raiz.join("pacote"));
    assert!(carga.invalidos.is_empty());
    let pacote = carga.pacotes.into_iter().next().unwrap();
    Cenario { _pasta: pasta, raiz, original, destino, pacote }
}

fn pedido(c: &Cenario, saida: PathBuf) -> Pedido {
    Pedido { iso_original: c.original.clone(), saida, sobrescrever: false, xdelta3: xdelta3() }
}

fn parciais(pasta: &Path) -> Vec<PathBuf> {
    fs::read_dir(pasta).unwrap().map(|e| e.unwrap().path()).filter(|p| p.to_string_lossy().ends_with(".parcial")).collect()
}

#[test]
fn gera_iso_identica_ao_destino() {
    let c = cenario("simples");
    let saida = c.raiz.join("saida.iso");
    let r = aplicar(&c.pacote, &pedido(&c, saida.clone()), &mut |_, _| {}, &AtomicBool::new(false)).unwrap();
    assert_eq!(fs::read(&saida).unwrap(), c.destino);
    assert!(r.compativel_ra);
    assert_eq!(r.saida, saida);
    assert!(parciais(&c.raiz).is_empty());
}

#[test]
fn informa_as_tres_etapas_e_progresso_chega_a_1() {
    let c = cenario("etapas");
    let mut vistos: Vec<(Etapa, f32)> = Vec::new();
    aplicar(&c.pacote, &pedido(&c, c.raiz.join("saida.iso")), &mut |e, f| vistos.push((e, f)), &AtomicBool::new(false)).unwrap();
    for etapa in [Etapa::Conferindo, Etapa::Aplicando, Etapa::Verificando] {
        assert!(vistos.iter().any(|(e, f)| *e == etapa && *f >= 1.0), "faltou {etapa:?} chegar a 1");
    }
    assert!(vistos.iter().all(|(_, f)| (0.0..=1.0).contains(f)));
    let primeira = |e: Etapa| vistos.iter().position(|(x, _)| *x == e).unwrap();
    assert!(primeira(Etapa::Conferindo) < primeira(Etapa::Aplicando));
    assert!(primeira(Etapa::Aplicando) < primeira(Etapa::Verificando));
}

#[test]
fn nome_de_saida() {
    let c = cenario("nome");
    assert_eq!(nome_saida(&c.pacote.ficha, &c.original), c.raiz.join("Jak II (USA) PT-BR v1.0.iso"));
}

#[test]
fn iso_de_outro_jogo_e_recusada() {
    let c = cenario("outro");
    let outra = comum::gravar(
        &c.raiz,
        "outro.iso",
        &comum::iso_sintetica(&[("SYSTEM.CNF", b"BOOT2 = cdrom0:\\SLES_123.45;1\r\n"), ("SLES_123.45", b"ELF")]),
    );
    match conferir(&c.pacote, &outra) {
        Err(Erro::JogoErrado { esperado_serial, encontrado_serial, esperado_nome }) => {
            assert_eq!(esperado_serial, "SCUS-97265");
            assert_eq!(encontrado_serial, "SLES-12345");
            assert_eq!(esperado_nome, "Jak II (USA)");
        }
        outro => panic!("esperava JogoErrado, veio {outro:?}"),
    }
    let mut p = pedido(&c, c.raiz.join("saida.iso"));
    p.iso_original = outra;
    assert!(matches!(aplicar(&c.pacote, &p, &mut |_, _| {}, &AtomicBool::new(false)), Err(Erro::JogoErrado { .. })));
    assert!(!c.raiz.join("saida.iso").exists());
}

#[test]
fn funciona_em_pasta_com_acento_e_espaco() {
    let c = cenario("Traduções\\Jak II");
    let saida = c.raiz.join("Saída com acento.iso");
    aplicar(&c.pacote, &pedido(&c, saida.clone()), &mut |_, _| {}, &AtomicBool::new(false)).unwrap();
    assert_eq!(fs::read(&saida).unwrap(), c.destino);
}

#[test]
fn saida_existente_sem_confirmacao_e_recusada() {
    let c = cenario("existe");
    let saida = comum::gravar(&c.raiz, "saida.iso", b"antigo");
    let r = aplicar(&c.pacote, &pedido(&c, saida.clone()), &mut |_, _| {}, &AtomicBool::new(false));
    assert!(matches!(r, Err(Erro::SaidaJaExiste(_))));
    assert_eq!(fs::read(&saida).unwrap(), b"antigo");

    let mut p = pedido(&c, saida.clone());
    p.sobrescrever = true;
    aplicar(&c.pacote, &p, &mut |_, _| {}, &AtomicBool::new(false)).unwrap();
    assert_eq!(fs::read(&saida).unwrap(), c.destino);
}

#[test]
fn patch_corrompido_apaga_parcial() {
    let c = cenario("corrompido");
    let patch = c.pacote.patch();
    let bytes = fs::read(&patch).unwrap();
    fs::write(&patch, &bytes[..bytes.len() / 2]).unwrap();
    let saida = c.raiz.join("saida.iso");
    let r = aplicar(&c.pacote, &pedido(&c, saida.clone()), &mut |_, _| {}, &AtomicBool::new(false));
    assert!(matches!(r, Err(Erro::XdeltaFalhou(_))), "{r:?}");
    assert!(!saida.exists());
    assert!(parciais(&c.raiz).is_empty());
}

#[test]
fn cancelar_apaga_parcial() {
    let c = cenario("cancelar");
    let saida = c.raiz.join("saida.iso");
    let r = aplicar(&c.pacote, &pedido(&c, saida.clone()), &mut |_, _| {}, &AtomicBool::new(true));
    assert!(matches!(r, Err(Erro::Cancelado)), "{r:?}");
    assert!(!saida.exists());
    assert!(parciais(&c.raiz).is_empty());
}

#[test]
fn saida_igual_a_original_e_recusada() {
    let c = cenario("igual");
    let mut p = pedido(&c, c.original.clone());
    p.sobrescrever = true;
    let antes = fs::read(&c.original).unwrap();
    let r = aplicar(&c.pacote, &p, &mut |_, _| {}, &AtomicBool::new(false));
    assert!(matches!(r, Err(Erro::SaidaIgualOriginal)), "{r:?}");
    assert_eq!(fs::read(&c.original).unwrap(), antes);
}

#[test]
fn ignora_variavel_xdelta() {
    // A variável XDELTA muda as opções padrão do xdelta3; o app não pode depender dela.
    unsafe { std::env::set_var("XDELTA", "-e") };
    let c = cenario("variavel");
    let saida = c.raiz.join("saida.iso");
    aplicar(&c.pacote, &pedido(&c, saida.clone()), &mut |_, _| {}, &AtomicBool::new(false)).unwrap();
    assert_eq!(fs::read(&saida).unwrap(), c.destino);
}
