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
    assert_eq!(Plataforma::de_id("wii"), None);
}

#[test]
fn ler_alvo_ps2() {
    let alvo = ler_alvo(&ficha(&jak2())).unwrap();
    let Alvo::Ps2 { origem, .. } = &alvo else { panic!("esperava PS2") };
    assert_eq!(origem.serial, "SCUS-97265");
    assert_eq!(alvo.plataforma(), Plataforma::Ps2);
    assert_eq!(alvo.detalhes(), "versão USA (SCUS-97265)");
    assert_eq!(alvo.instrucao(), "Use a sua ISO original da versão USA. Ela não é alterada: a traduzida é gravada ao lado.");
    assert_eq!(alvo.tamanho_resultado(), 100);
    assert_eq!(alvo.hash_ra_esperado(), Some(comum::HASH_RA));
}

#[test]
fn ler_alvo_plataforma_desconhecida() {
    let erro = ler_alvo(&ficha(&jak2().replace(r#""plataforma":"ps2""#, r#""plataforma":"wii""#))).unwrap_err();
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

fn ps1() -> String {
    comum::ficha_ps1("jogo-ps1-a", comum::HASH_RA, comum::HASH_RA, 2352 * 30, 2352 * 30, &"a".repeat(64), 10, "")
}

#[test]
fn ps1_conhecida_e_info() {
    assert_eq!(Plataforma::de_id("ps1"), Some(Plataforma::Ps1));
    let info = Plataforma::Ps1.info();
    assert_eq!(info.nome, "PlayStation");
    assert_eq!(info.sigla, "PS1");
    assert_eq!(info.arquivo, "imagem (.cue)");
    assert_eq!(info.extensoes, &["cue"]);
    assert!(info.tem_ra);
    assert!(Plataforma::Ps2.ordem() < Plataforma::Ps1.ordem());
}

#[test]
fn ficha_ps1_lida() {
    let alvo = ler_alvo(&ficha(&ps1())).unwrap();
    let Alvo::Ps1 { origem, resultado } = &alvo else { panic!("esperava PS1") };
    assert_eq!(origem.serial, "SLUS-00684");
    assert_eq!(resultado.tamanho, 2352 * 30);
    assert_eq!(alvo.plataforma(), Plataforma::Ps1);
    assert_eq!(alvo.detalhes(), "versão USA (SLUS-00684)");
    assert_eq!(alvo.hash_ra_esperado(), Some(comum::HASH_RA));
    assert!(alvo.url_capa(None).unwrap().ends_with("/psx-covers/main/covers/default/SLUS-00684.jpg"));
}

#[test]
fn ficha_ps1_hash_curto() {
    let curto = &comum::HASH_RA[..31];
    let erro = ler_alvo(&ficha(&ps1().replacen(comum::HASH_RA, curto, 1))).unwrap_err();
    assert_eq!(erro, "origem.hash_ra deve ter 32 caracteres hexadecimais");
}

#[test]
fn pc_conhecida_e_info() {
    assert_eq!(Plataforma::de_id("pc"), Some(Plataforma::Pc));
    let info = Plataforma::Pc.info();
    assert_eq!(info.nome, "PC");
    assert_eq!(info.sigla, "PC");
    assert_eq!(info.arquivo, "pasta do jogo");
    assert!(info.extensoes.is_empty());
    assert!(!info.tem_ra);
    assert!(info.escolhe_pasta);
    assert!(!Plataforma::Ps2.info().escolhe_pasta);
    assert!(!Plataforma::Ps1.info().escolhe_pasta);
    assert!(Plataforma::Ps1.ordem() < Plataforma::Pc.ordem());
}

#[test]
fn ficha_pc_lida() {
    let alvo = ler_alvo(&ficha(&comum::ficha_pc_simples(&["mods/textos.sub"]))).unwrap();
    let Alvo::Pc { origem } = &alvo else { panic!("esperava PC") };
    assert_eq!(origem.loja, "Steam");
    assert_eq!(origem.versao, "1.0.3");
    assert_eq!(origem.referencias.len(), 1);
    assert_eq!(alvo.plataforma(), Plataforma::Pc);
    assert_eq!(alvo.hash_ra_esperado(), None);
    assert_eq!(alvo.url_capa(None), None);
    assert_eq!(alvo.detalhes(), "versão 1.0.3 (Steam)");
}

#[test]
fn cada_plataforma_tem_a_proporcao_da_caixa() {
    assert_eq!(Plataforma::Ps2.info().proporcao_capa, "7/10");
    assert_eq!(Plataforma::Ps1.info().proporcao_capa, "1/1");
    assert_eq!(Plataforma::Pc.info().proporcao_capa, "2/3");
}

fn pc_da_steam() -> String {
    comum::ficha_pc_simples(&["a.txt"]).replace(r#""loja":"Steam","#, r#""loja":"Steam","steam_app_id":758330,"#)
}

#[test]
fn capa_do_pc_vem_da_steam_pelo_id() {
    let alvo = ler_alvo(&ficha(&pc_da_steam())).unwrap();
    let Alvo::Pc { origem } = &alvo else { panic!("esperava PC") };
    assert_eq!(origem.steam_app_id, Some(758330));
    assert_eq!(
        alvo.url_capa(None).as_deref(),
        Some("https://cdn.cloudflare.steamstatic.com/steam/apps/758330/library_600x900.jpg")
    );
    assert_eq!(alvo.url_capa(Some("http://x/{serial}.jpg")).as_deref(), Some("http://x/758330.jpg"));
}

#[test]
fn pc_sem_id_da_steam_fica_sem_capa_do_acervo() {
    let alvo = ler_alvo(&ficha(&comum::ficha_pc_simples(&["a.txt"]))).unwrap();
    assert_eq!(alvo.url_capa(None), None);
}
