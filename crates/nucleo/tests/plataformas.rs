mod comum;

use nucleo::{Alvo, Ficha, Plataforma, ler_alvo};

fn ficha(json: &str) -> Ficha {
    serde_json::from_str(json).unwrap()
}

fn jak2() -> String {
    comum::ficha_catalogo("jak2-a", "1.0", "https://x", comum::HASH_RA, 100, b"PATCH", None)
}

#[test]
fn ps2_conhecida_e_info() {
    assert_eq!(Plataforma::de_id("ps2"), Some(Plataforma::Ps2));
    let info = Plataforma::Ps2.info();
    assert_eq!(info.id, "ps2");
    assert_eq!(info.nome, "PlayStation 2");
    assert_eq!(info.sigla, "PS2");
    assert_eq!(info.arquivo, "ISO");
    assert_eq!(info.extensoes, &["iso"]);
    assert!(info.tem_ra);
    assert_eq!(Plataforma::de_id("pc"), None);
}

#[test]
fn ler_alvo_ps2() {
    let alvo = ler_alvo(&ficha(&jak2())).unwrap();
    let Alvo::Ps2 { origem, .. } = &alvo;
    assert_eq!(origem.serial, "SCUS-97265");
    assert_eq!(alvo.plataforma(), Plataforma::Ps2);
    assert_eq!(alvo.detalhes(), "versão USA (SCUS-97265)");
    assert_eq!(alvo.instrucao(), "Use a sua ISO original da versão USA. Ela não é alterada: a traduzida é gravada ao lado.");
    assert_eq!(alvo.tamanho_resultado(), 100);
    assert_eq!(alvo.hash_ra_esperado(), Some(comum::HASH_RA));
}

#[test]
fn ler_alvo_plataforma_desconhecida() {
    let erro = ler_alvo(&ficha(&jak2().replace(r#""plataforma":"ps2""#, r#""plataforma":"pc""#))).unwrap_err();
    assert!(erro.contains("plataforma desconhecida"), "{erro}");
}

#[test]
fn ler_alvo_origem_sem_serial() {
    assert!(ler_alvo(&ficha(&jak2().replace(r#""serial":"SCUS-97265","#, ""))).is_err());
}

#[test]
fn ler_alvo_hash_mal_formado() {
    assert!(ler_alvo(&ficha(&jak2().replace(comum::HASH_RA, "1234"))).is_err());
}

#[test]
fn url_capa_ps2() {
    let alvo = ler_alvo(&ficha(&jak2())).unwrap();
    assert!(alvo.url_capa(None).unwrap().ends_with("/SCUS-97265.jpg"));
    assert_eq!(alvo.url_capa(Some("http://x/{serial}.png")).as_deref(), Some("http://x/SCUS-97265.png"));
}
