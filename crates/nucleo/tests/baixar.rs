mod comum;

use std::fs;
use std::sync::atomic::AtomicBool;

use comum::servidor::{Resposta, Servidor};
use comum::{HASH_RA, ficha_catalogo, indice};
use nucleo::{Erro, Pacote, RedeUreq, baixar_patch, carregar_pacotes, sincronizar};

const PATCH: &[u8] = b"conteudo do patch xdelta de teste, com alguns bytes a mais para o corte no meio";

fn preparar() -> (Servidor, tempfile::TempDir, Pacote) {
    let s = Servidor::novo();
    let fichas = vec![ficha_catalogo("jak2-a", "1.0", &s.url(""), HASH_RA, 100, PATCH, None)];
    s.definir("/indice.json", Resposta::Ok(indice(&fichas)));
    s.definir("/jak2-a/patch", Resposta::Ok(PATCH.to_vec()));
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    let pacote = carregar_pacotes(cache.path()).pacotes.remove(0);
    (s, cache, pacote)
}

fn parciais(pacote: &Pacote) -> usize {
    fs::read_dir(&pacote.pasta).unwrap().filter(|e| e.as_ref().unwrap().path().to_string_lossy().ends_with(".parcial")).count()
}

#[test]
fn baixa_confere_e_guarda_o_patch() {
    let (_s, _c, pacote) = preparar();
    let mut visto = Vec::new();
    baixar_patch(&RedeUreq::novo(), &pacote, &mut |f| visto.push(f), &AtomicBool::new(false)).unwrap();
    assert!(pacote.patch_disponivel());
    assert_eq!(fs::read(pacote.patch()).unwrap(), PATCH);
    assert!(visto.last().copied().unwrap_or(0.0) >= 1.0);
}

#[test]
fn sha_diferente_apaga_e_recusa() {
    let (s, _c, pacote) = preparar();
    let mut trocado = PATCH.to_vec();
    trocado[0] ^= 0xff;
    s.definir("/jak2-a/patch", Resposta::Ok(trocado));
    let r = baixar_patch(&RedeUreq::novo(), &pacote, &mut |_| {}, &AtomicBool::new(false));
    assert!(matches!(r, Err(Erro::DownloadInvalido(_))), "{r:?}");
    assert!(!pacote.patch_disponivel());
    assert_eq!(parciais(&pacote), 0);
}

#[test]
fn conexao_cortada_apaga_o_parcial() {
    let (s, _c, pacote) = preparar();
    s.definir("/jak2-a/patch", Resposta::Cortar(PATCH.to_vec()));
    assert!(baixar_patch(&RedeUreq::novo(), &pacote, &mut |_| {}, &AtomicBool::new(false)).is_err());
    assert!(!pacote.patch_disponivel());
    assert_eq!(parciais(&pacote), 0);
    s.definir("/jak2-a/patch", Resposta::Ok(PATCH.to_vec()));
    baixar_patch(&RedeUreq::novo(), &pacote, &mut |_| {}, &AtomicBool::new(false)).unwrap();
    assert!(pacote.patch_disponivel());
}

#[test]
fn patch_ja_baixado_nao_baixa_de_novo() {
    let (s, _c, pacote) = preparar();
    baixar_patch(&RedeUreq::novo(), &pacote, &mut |_| {}, &AtomicBool::new(false)).unwrap();
    baixar_patch(&RedeUreq::novo(), &pacote, &mut |_| {}, &AtomicBool::new(false)).unwrap();
    assert_eq!(s.pedidos("/jak2-a/patch"), 1);
}

#[test]
fn cancelar_durante_o_download() {
    let (_s, _c, pacote) = preparar();
    let r = baixar_patch(&RedeUreq::novo(), &pacote, &mut |_| {}, &AtomicBool::new(true));
    assert!(matches!(r, Err(Erro::Cancelado)), "{r:?}");
    assert!(!pacote.patch_disponivel());
    assert_eq!(parciais(&pacote), 0);
}
