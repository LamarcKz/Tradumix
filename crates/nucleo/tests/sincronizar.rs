mod comum;

use std::fs;

use comum::servidor::{Resposta, Servidor};
use comum::{HASH_RA, ficha_catalogo, indice};
use nucleo::{Historico, RedeUreq, carregar_pacotes, sincronizar};

const CAPA: &[u8] = b"imagem-da-capa";

fn servidor_com(ids: &[&str]) -> Servidor {
    let s = Servidor::novo();
    let base = s.url("");
    let fichas: Vec<String> = ids.iter().map(|id| ficha_catalogo(id, "1.0", &base, HASH_RA, 100, b"PATCH", Some(CAPA))).collect();
    s.definir("/indice.json", Resposta::Ok(indice(&fichas)));
    for id in ids {
        s.definir(&format!("/{id}/capa"), Resposta::Ok(CAPA.to_vec()));
    }
    s
}

#[test]
fn sincroniza_e_grava_fichas_e_imagens() {
    let s = servidor_com(&["jak2-a", "jak2-b"]);
    let cache = tempfile::tempdir().unwrap();
    let r = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    assert!(!r.offline, "{:?}", r.aviso);
    assert!(r.aviso.is_none());
    assert!(r.atualizado_em.is_some());
    let carga = carregar_pacotes(cache.path());
    assert_eq!(carga.pacotes.len(), 2, "{:?}", carga.invalidos);
    assert_eq!(fs::read(cache.path().join("jak2-a").join("capa.jpg")).unwrap(), CAPA);
    assert!(!carga.pacotes[0].patch_disponivel(), "o patch só é baixado na hora de traduzir");
}

#[test]
fn indice_quebrado_mantem_o_cache_anterior() {
    let s = servidor_com(&["jak2-a"]);
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    s.definir("/indice.json", Resposta::Ok(b"{ isto quebrou".to_vec()));
    let r = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    assert!(r.aviso.unwrap().contains("anterior"));
    assert_eq!(carregar_pacotes(cache.path()).pacotes.len(), 1);
}

#[test]
fn indice_cortado_no_meio_mantem_o_cache_anterior() {
    let s = servidor_com(&["jak2-a"]);
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    let fichas = vec![ficha_catalogo("jak2-a", "1.0", &s.url(""), HASH_RA, 100, b"PATCH", None)];
    s.definir("/indice.json", Resposta::Cortar(indice(&fichas)));
    let r = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    assert!(r.aviso.is_some());
    assert_eq!(carregar_pacotes(cache.path()).pacotes.len(), 1);
}

#[test]
fn servidor_fora_do_ar_fica_offline_com_o_cache() {
    let s = servidor_com(&["jak2-a"]);
    let cache = tempfile::tempdir().unwrap();
    let primeira = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    let r = sincronizar(&RedeUreq::novo(), "http://127.0.0.1:9/indice.json", cache.path(), true);
    assert!(r.offline);
    assert_eq!(r.atualizado_em, primeira.atualizado_em);
    assert!(r.aviso.unwrap().starts_with("Sem conexão"));
    assert_eq!(carregar_pacotes(cache.path()).pacotes.len(), 1);
}

#[test]
fn traducao_removida_some_do_cache() {
    let s = servidor_com(&["jak2-a", "jak2-b"]);
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    let antes = carregar_pacotes(cache.path());
    let mut historico = Historico::default();
    let b = antes.pacotes.iter().find(|p| p.ficha.id == "jak2-b").unwrap();
    historico.registrar(b, &nucleo::Resultado { saida: "x.iso".into(), hash_ra: HASH_RA.into(), compativel_ra: true, segundos: 1.0 }, "2026-11-05");

    let fichas = vec![ficha_catalogo("jak2-a", "1.0", &s.url(""), HASH_RA, 100, b"PATCH", Some(CAPA))];
    s.definir("/indice.json", Resposta::Ok(indice(&fichas)));
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    let depois = carregar_pacotes(cache.path());
    assert_eq!(depois.pacotes.len(), 1);
    assert!(!cache.path().join("jak2-b").exists());
    assert!(historico.atualizacoes(&depois.pacotes).is_empty());
}

#[test]
fn imagem_com_sha_errado_e_descartada_e_o_resto_sincroniza() {
    let s = servidor_com(&["jak2-a"]);
    s.definir("/jak2-a/capa", Resposta::Ok(b"outra imagem".to_vec()));
    let cache = tempfile::tempdir().unwrap();
    let r = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    assert!(!r.offline);
    assert!(!cache.path().join("jak2-a").join("capa.jpg").exists());
    assert_eq!(carregar_pacotes(cache.path()).pacotes.len(), 1);
}

#[test]
fn nao_baixa_de_novo_imagem_que_nao_mudou() {
    let s = servidor_com(&["jak2-a"]);
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    assert_eq!(s.pedidos("/jak2-a/capa"), 1);
    assert_eq!(s.pedidos("/indice.json"), 2);
}

#[test]
fn versao_nova_apaga_o_patch_antigo() {
    let s = servidor_com(&["jak2-a"]);
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    fs::write(cache.path().join("jak2-a").join("patch.xdelta"), b"PATCH").unwrap();
    let fichas = vec![ficha_catalogo("jak2-a", "1.1", &s.url(""), HASH_RA, 100, b"PATCH NOVO", Some(CAPA))];
    s.definir("/indice.json", Resposta::Ok(indice(&fichas)));
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true);
    assert!(!cache.path().join("jak2-a").join("patch.xdelta").exists());
}
