//! Testes com o Jak II de verdade. Ficam fora do `cargo test` normal porque precisam das ISOs:
//!   TRADUMIX_PATCH_JAK2 o .xdelta da tradução do Jak II (o da release do catálogo)
//!   TRADUMIX_ISO_JAK2 ISO do Jak II (USA)
//!   TRADUMIX_SAIDA    pasta onde gravar a ISO gerada (só no teste de ponta a ponta)
//!   TRADUMIX_CATALOGO pasta do catálogo, com o indice-v2.json (só no teste do catálogo)
//! Rodar com: cargo test -p nucleo --test jak2_real -- --ignored

mod comum;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use md5::{Digest, Md5};
use nucleo::{Pedido, aplicar, carregar_pacotes, conferir};

fn var(nome: &str) -> PathBuf {
    PathBuf::from(std::env::var(nome).unwrap_or_else(|_| panic!("defina {nome}")))
}

/// Monta o pacote do Jak II (ficha formato 2, como no catálogo) numa pasta temporária, com o patch real.
fn pacote_jak2() -> (tempfile::TempDir, nucleo::Pacote) {
    let patch = std::fs::read(var("TRADUMIX_PATCH_JAK2")).unwrap();
    let raiz = tempfile::tempdir().unwrap();
    let pasta = raiz.path().join("jak2-usa-ptbr");
    std::fs::create_dir_all(&pasta).unwrap();
    std::fs::write(pasta.join("patch.xdelta"), &patch).unwrap();
    let hash = "f51c94dd47cc2eb385d5733389428121";
    let ficha = comum::ficha_ps2(
        "jak2-usa-ptbr",
        "1.0",
        hash,
        4_410_671_104,
        4_410_671_104,
        "https://x/patch.xdelta",
        &comum::sha256_hex(&patch),
        patch.len() as u64,
        "",
    );
    std::fs::write(pasta.join("ficha.json"), ficha).unwrap();
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.invalidos.is_empty(), "{:?}", carga.invalidos);
    let pacote = carga.pacotes.into_iter().next().expect("pacote jak2-usa-ptbr");
    (raiz, pacote)
}

/// Serve a pasta do catálogo (`TRADUMIX_CATALOGO`) num servidor local, trocando as URLs do GitHub
/// pelas dele, e confere que o app sincroniza o Jak II (capa pelo serial, link do trailer) e baixa
/// o patch com o SHA-256 certo.
#[test]
#[ignore = "precisa da pasta do catálogo em TRADUMIX_CATALOGO"]
fn catalogo_local_ponta_a_ponta() {
    use comum::servidor::{Resposta, Servidor};
    let pasta = var("TRADUMIX_CATALOGO");
    let s = Servidor::novo();
    let indice = std::fs::read_to_string(pasta.join("indice-v2.json"))
        .unwrap()
        .replace("https://raw.githubusercontent.com/LamarcKz/Tradumix-catalogo/main", &s.url(""))
        .replace(
            "https://github.com/LamarcKz/Tradumix-catalogo/releases/download/jak2-usa-ptbr-v1.0",
            &s.url("/release"),
        );
    s.definir("/indice.json", Resposta::Ok(indice.into_bytes()));
    // A capa vem do acervo pelo serial; aqui ele é simulado pelo servidor local.
    s.definir("/capas/SCUS-97265.jpg", Resposta::Ok(vec![0xFF, 0xD8, 0xFF, 0xE0, 1, 2, 3]));
    s.definir("/release/jak2-usa-ptbr-v1.0.xdelta", Resposta::Ok(std::fs::read(pasta.join("release/jak2-usa-ptbr-v1.0.xdelta")).unwrap()));

    let cache = tempfile::tempdir().unwrap();
    let capas = s.url("/capas/{serial}.jpg");
    let r = nucleo::sincronizar(&nucleo::RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, nucleo::Capas::Modelo(capas));
    assert!(!r.offline && r.aviso.is_none(), "{:?}", r.aviso);
    let pacote = carregar_pacotes(cache.path()).pacotes.into_iter().find(|p| p.ficha.id == "jak2-usa-ptbr").unwrap();
    assert!(pacote.imagem(nucleo::Imagem::Capa).unwrap().is_file());
    assert_eq!(pacote.ficha.midia.as_ref().unwrap().trailer_youtube.as_deref(), Some("Pj-ueiUYEbg"));
    assert!(!pacote.patch_disponivel());
    nucleo::baixar_patch(&nucleo::RedeUreq::novo(), &pacote, &mut |_| {}, &AtomicBool::new(false)).unwrap();
    assert!(pacote.patch_disponivel());
    assert_eq!(std::fs::metadata(pacote.patch()).unwrap().len(), 238_940);
}

#[test]
#[ignore = "precisa do pacote e da ISO do Jak II"]
fn jak2_pacote_real_e_reconhecido() {
    let (_raiz, pacote) = pacote_jak2();
    conferir(&pacote, &var("TRADUMIX_ISO_JAK2")).unwrap();
}

#[test]
#[ignore = "precisa da ISO original do Jak II e de 4,4 GB livres"]
fn jak2_ponta_a_ponta() {
    let (_raiz, pacote) = pacote_jak2();
    let saida = var("TRADUMIX_SAIDA").join("Jak II (USA) PT-BR v1.0.iso");
    let xdelta3 = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/binaries/xdelta3-x86_64-pc-windows-msvc.exe");
    let pedido = Pedido { iso_original: var("TRADUMIX_ISO_JAK2"), saida: saida.clone(), sobrescrever: true, xdelta3 };
    let r = aplicar(&pacote, &pedido, &mut |_, _| {}, &AtomicBool::new(false)).unwrap();
    assert_eq!(r.compativel_ra, Some(true));
    let mut md5 = Md5::new();
    let mut arquivo = File::open(&saida).unwrap();
    let mut buf = vec![0u8; 8 << 20];
    loop {
        let n = arquivo.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        md5.update(&buf[..n]);
    }
    let hex: String = md5.finalize().iter().map(|b| format!("{b:02x}")).collect();
    std::fs::remove_file(&saida).unwrap();
    assert_eq!(hex, "931f3f2cf436e2aa231c76461ab0bdac");
}
