mod comum;

use std::fs;

use comum::servidor::{Resposta, Servidor};
use comum::{HASH_RA, ficha_catalogo, indice};
use nucleo::{Capas, Historico, RedeUreq, carregar_pacotes, sincronizar};

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
    let r = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
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
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    s.definir("/indice.json", Resposta::Ok(b"{ isto quebrou".to_vec()));
    let r = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    assert!(r.aviso.unwrap().contains("anterior"));
    assert_eq!(carregar_pacotes(cache.path()).pacotes.len(), 1);
}

#[test]
fn indice_cortado_no_meio_mantem_o_cache_anterior() {
    let s = servidor_com(&["jak2-a"]);
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    let fichas = vec![ficha_catalogo("jak2-a", "1.0", &s.url(""), HASH_RA, 100, b"PATCH", None)];
    s.definir("/indice.json", Resposta::Cortar(indice(&fichas)));
    let r = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    assert!(r.aviso.is_some());
    assert_eq!(carregar_pacotes(cache.path()).pacotes.len(), 1);
}

#[test]
fn servidor_fora_do_ar_fica_offline_com_o_cache() {
    let s = servidor_com(&["jak2-a"]);
    let cache = tempfile::tempdir().unwrap();
    let primeira = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    let r = sincronizar(&RedeUreq::novo(), "http://127.0.0.1:9/indice.json", cache.path(), true, Capas::Nao);
    assert!(r.offline);
    assert_eq!(r.atualizado_em, primeira.atualizado_em);
    assert!(r.aviso.unwrap().starts_with("Sem conexão"));
    assert_eq!(carregar_pacotes(cache.path()).pacotes.len(), 1);
}

#[test]
fn traducao_removida_some_do_cache() {
    let s = servidor_com(&["jak2-a", "jak2-b"]);
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    let antes = carregar_pacotes(cache.path());
    let mut historico = Historico::default();
    let b = antes.pacotes.iter().find(|p| p.ficha.id == "jak2-b").unwrap();
    historico.registrar(b, &nucleo::Resultado { saida: "x.iso".into(), hash_ra: Some(HASH_RA.into()), compativel_ra: Some(true), segundos: 1.0 }, "2026-11-05");

    let fichas = vec![ficha_catalogo("jak2-a", "1.0", &s.url(""), HASH_RA, 100, b"PATCH", Some(CAPA))];
    s.definir("/indice.json", Resposta::Ok(indice(&fichas)));
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
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
    let r = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    assert!(!r.offline);
    assert!(!cache.path().join("jak2-a").join("capa.jpg").exists());
    assert_eq!(carregar_pacotes(cache.path()).pacotes.len(), 1);
}

#[test]
fn nao_baixa_de_novo_imagem_que_nao_mudou() {
    let s = servidor_com(&["jak2-a"]);
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    assert_eq!(s.pedidos("/jak2-a/capa"), 1);
    assert_eq!(s.pedidos("/indice.json"), 2);
}

#[test]
fn versao_nova_apaga_o_patch_antigo() {
    let s = servidor_com(&["jak2-a"]);
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    fs::write(cache.path().join("jak2-a").join("patch.xdelta"), b"PATCH").unwrap();
    let fichas = vec![ficha_catalogo("jak2-a", "1.1", &s.url(""), HASH_RA, 100, b"PATCH NOVO", Some(CAPA))];
    s.definir("/indice.json", Resposta::Ok(indice(&fichas)));
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    assert!(!cache.path().join("jak2-a").join("patch.xdelta").exists());
}

const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0x10, b'J', b'F', b'I', b'F', 0, 1];

/// Ficha do catálogo sem capa própria, com o trailer do YouTube.
fn ficha_sem_capa(s: &Servidor, id: &str) -> String {
    let mut f = ficha_catalogo(id, "1.0", &s.url(""), HASH_RA, 100, b"PATCH", None);
    f.pop();
    f + r#","midia":{"trailer_youtube":"Pj-ueiUYEbg"}}"#
}

fn sincronizar_com_capas(s: &Servidor, cache: &std::path::Path) {
    let modelo = s.url("/capas/{serial}.jpg");
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache, true, Capas::Modelo(modelo));
}

#[test]
fn busca_a_capa_pelo_serial_quando_a_ficha_nao_tem() {
    let s = Servidor::novo();
    s.definir("/indice.json", Resposta::Ok(indice(&[ficha_sem_capa(&s, "jak2-a")])));
    s.definir("/capas/SCUS-97265.jpg", Resposta::Ok(JPEG.to_vec()));
    let cache = tempfile::tempdir().unwrap();
    sincronizar_com_capas(&s, cache.path());
    let p = carregar_pacotes(cache.path()).pacotes.remove(0);
    assert_eq!(fs::read(p.imagem(nucleo::Imagem::Capa).unwrap()).unwrap(), JPEG);
    assert_eq!(p.ficha.midia.as_ref().unwrap().trailer_youtube.as_deref(), Some("Pj-ueiUYEbg"));
}

#[test]
fn capa_que_nao_existe_no_acervo_fica_sem_capa() {
    let s = Servidor::novo();
    s.definir("/indice.json", Resposta::Ok(indice(&[ficha_sem_capa(&s, "jak2-a")])));
    let cache = tempfile::tempdir().unwrap();
    sincronizar_com_capas(&s, cache.path());
    let p = carregar_pacotes(cache.path()).pacotes.remove(0);
    assert!(p.imagem(nucleo::Imagem::Capa).is_none());
}

#[test]
fn capa_que_nao_e_imagem_e_descartada() {
    let s = Servidor::novo();
    s.definir("/indice.json", Resposta::Ok(indice(&[ficha_sem_capa(&s, "jak2-a")])));
    s.definir("/capas/SCUS-97265.jpg", Resposta::Ok(b"<html>nao e imagem</html>".to_vec()));
    let cache = tempfile::tempdir().unwrap();
    sincronizar_com_capas(&s, cache.path());
    let p = carregar_pacotes(cache.path()).pacotes.remove(0);
    assert!(p.imagem(nucleo::Imagem::Capa).is_none());
    assert!(!cache.path().join("jak2-a").join("capa.jpg").exists());
}

#[test]
fn nao_busca_a_capa_do_acervo_de_novo() {
    let s = Servidor::novo();
    s.definir("/indice.json", Resposta::Ok(indice(&[ficha_sem_capa(&s, "jak2-a")])));
    s.definir("/capas/SCUS-97265.jpg", Resposta::Ok(JPEG.to_vec()));
    let cache = tempfile::tempdir().unwrap();
    sincronizar_com_capas(&s, cache.path());
    sincronizar_com_capas(&s, cache.path());
    assert_eq!(s.pedidos("/capas/SCUS-97265.jpg"), 1);
    assert!(carregar_pacotes(cache.path()).pacotes[0].imagem(nucleo::Imagem::Capa).is_some());
}

#[test]
fn cache_com_ficha_formato_1_e_ignorado() {
    let cache = tempfile::tempdir().unwrap();
    let velho = cache.path().join("jak2-velho");
    fs::create_dir_all(&velho).unwrap();
    fs::write(velho.join("ficha.json"), r#"{"formato":1,"id":"jak2-velho","jogo":{"id":"jak2","nome":"Jak II","console":"ps2"}}"#).unwrap();
    assert!(carregar_pacotes(cache.path()).pacotes.is_empty());
    let s = servidor_com(&["jak2-a"]);
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    assert!(!velho.exists());
    let carga = carregar_pacotes(cache.path());
    assert_eq!(carga.pacotes.len(), 1);
    assert_eq!(carga.pacotes[0].ficha.id, "jak2-a");
}

#[test]
fn ficha_que_ficou_invalida_sai_do_cache() {
    let s = servidor_com(&["jak2-a"]);
    let cache = tempfile::tempdir().unwrap();
    sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    assert!(cache.path().join("jak2-a").exists());
    let pc = ficha_catalogo("jak2-a", "1.1", &s.url(""), HASH_RA, 100, b"PATCH", None).replace(r#""plataforma":"ps2""#, r#""plataforma":"pc""#);
    s.definir("/indice.json", Resposta::Ok(indice(&[pc])));
    let r = sincronizar(&RedeUreq::novo(), &s.url("/indice.json"), cache.path(), true, Capas::Nao);
    assert!(!r.offline && r.aviso.is_none(), "{:?}", r.aviso);
    assert!(!cache.path().join("jak2-a").exists());
}

#[test]
fn capa_do_acervo_nao_reescreve_a_ficha() {
    let s = Servidor::novo();
    s.definir("/indice.json", Resposta::Ok(indice(&[ficha_sem_capa(&s, "jak2-a")])));
    s.definir("/capas/SCUS-97265.jpg", Resposta::Ok(JPEG.to_vec()));
    let cache = tempfile::tempdir().unwrap();
    sincronizar_com_capas(&s, cache.path());
    assert!(cache.path().join("jak2-a").join("capa.jpg").is_file());
    let ficha = fs::read_to_string(cache.path().join("jak2-a").join("ficha.json")).unwrap();
    assert!(!ficha.contains("imagens"), "{ficha}");
}
