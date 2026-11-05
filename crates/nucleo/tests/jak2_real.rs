//! Testes com o Jak II de verdade. Ficam fora do `cargo test` normal porque precisam das ISOs:
//!   TRADUMIX_PACOTES  pasta com o pacote de rascunho do Jak II
//!   TRADUMIX_ISO_JAK2 ISO do Jak II (USA)
//!   TRADUMIX_SAIDA    pasta onde gravar a ISO gerada (só no teste de ponta a ponta)
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

fn pacote_jak2() -> nucleo::Pacote {
    let carga = carregar_pacotes(&var("TRADUMIX_PACOTES"));
    assert!(carga.invalidos.is_empty(), "{:?}", carga.invalidos);
    carga.pacotes.into_iter().find(|p| p.ficha.id == "jak2-usa-ptbr").expect("pacote jak2-usa-ptbr")
}

/// Serve a pasta do catálogo (`TRADUMIX_CATALOGO`) num servidor local, trocando as URLs do GitHub
/// pelas dele, e confere que o app sincroniza o Jak II e baixa o patch com o SHA-256 certo.
#[test]
#[ignore = "precisa da pasta do catálogo em TRADUMIX_CATALOGO"]
fn catalogo_local_ponta_a_ponta() {
    use comum::servidor::{Resposta, Servidor};
    let pasta = var("TRADUMIX_CATALOGO");
    let s = Servidor::novo();
    let indice = std::fs::read_to_string(pasta.join("indice.json"))
        .unwrap()
        .replace("https://raw.githubusercontent.com/LamarcKz/Tradumix-catalogo/main", &s.url(""))
        .replace(
            "https://github.com/LamarcKz/Tradumix-catalogo/releases/download/jak2-usa-ptbr-v1.0",
            &s.url("/release"),
        );
    s.definir("/indice.json", Resposta::Ok(indice.into_bytes()));
    for arquivo in ["jogos/jak2/capa.jpg", "jogos/jak2/fundo.jpg"] {
        s.definir(&format!("/{arquivo}"), Resposta::Ok(std::fs::read(pasta.join(arquivo)).unwrap()));
    }
    s.definir("/release/jak2-usa-ptbr-v1.0.xdelta", Resposta::Ok(std::fs::read(pasta.join("release/jak2-usa-ptbr-v1.0.xdelta")).unwrap()));

    let cache = tempfile::tempdir().unwrap();
    let r = nucleo::sincronizar(&nucleo::RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, None);
    assert!(!r.offline && r.aviso.is_none(), "{:?}", r.aviso);
    let pacote = carregar_pacotes(cache.path()).pacotes.into_iter().find(|p| p.ficha.id == "jak2-usa-ptbr").unwrap();
    assert!(pacote.imagem(nucleo::Imagem::Capa).unwrap().is_file());
    assert!(pacote.imagem(nucleo::Imagem::Fundo).unwrap().is_file());
    assert!(!pacote.patch_disponivel());
    nucleo::baixar_patch(&nucleo::RedeUreq::novo(), &pacote, &mut |_| {}, &AtomicBool::new(false)).unwrap();
    assert!(pacote.patch_disponivel());
    assert_eq!(std::fs::metadata(pacote.patch()).unwrap().len(), 238_940);
}

#[test]
#[ignore = "precisa do pacote e da ISO do Jak II"]
fn jak2_pacote_real_e_reconhecido() {
    let pacote = pacote_jak2();
    let id = conferir(&pacote, &var("TRADUMIX_ISO_JAK2")).unwrap();
    assert_eq!(id.serial, "SCUS-97265");
}

#[test]
#[ignore = "precisa da ISO original do Jak II e de 4,4 GB livres"]
fn jak2_ponta_a_ponta() {
    let pacote = pacote_jak2();
    let saida = var("TRADUMIX_SAIDA").join("Jak II (USA) PT-BR v1.0.iso");
    let xdelta3 = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/binaries/xdelta3-x86_64-pc-windows-msvc.exe");
    let pedido = Pedido { iso_original: var("TRADUMIX_ISO_JAK2"), saida: saida.clone(), sobrescrever: true, xdelta3 };
    let r = aplicar(&pacote, &pedido, &mut |_, _| {}, &AtomicBool::new(false)).unwrap();
    assert!(r.compativel_ra);
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
