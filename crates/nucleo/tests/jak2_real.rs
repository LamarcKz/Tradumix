//! Testes com o Jak II de verdade. Ficam fora do `cargo test` normal porque precisam das ISOs:
//!   TRADUMIX_PACOTES  pasta com o pacote de rascunho do Jak II
//!   TRADUMIX_ISO_JAK2 ISO do Jak II (USA)
//!   TRADUMIX_SAIDA    pasta onde gravar a ISO gerada (só no teste de ponta a ponta)
//! Rodar com: cargo test -p nucleo --test jak2_real -- --ignored

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
