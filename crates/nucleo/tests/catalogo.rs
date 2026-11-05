use nucleo::{Erro, ler_indice};

const HASH: &str = "f51c94dd47cc2eb385d5733389428121";
const SHA: &str = "3ce44ebfecc30fc110c57f4fdf6ec0cd8c483fb3403d60ef3a671afb5f26e8de";

pub fn ficha(id: &str, url: &str, sha: &str, com_download: bool) -> String {
    let download = if com_download {
        format!(r#","download":{{"patch":{{"url":"{url}","sha256":"{sha}","tamanho":238940}},"capa":{{"url":"{url}.jpg","sha256":"{sha}","tamanho":10}}}}"#)
    } else {
        String::new()
    };
    format!(
        r#"{{"formato":1,"id":"{id}","jogo":{{"id":"jak2","nome":"Jak II","console":"ps2"}},"versao":"1.0","estado":"beta",
        "idioma":"pt-BR","data":"2026-11-05",
        "origem":{{"serial":"SCUS-97265","regiao":"usa","hash_ra":"{HASH}","tamanho":4410671104}},
        "resultado":{{"hash_ra":"{HASH}","tamanho":4410671104}},"tradutores":[{{"nome":"Fulano"}}],
        "arquivos":{{"patch":"patch.xdelta","capa":"capa.jpg"}}{download}}}"#
    )
}

fn indice(formato: u32, fichas: &[String]) -> String {
    format!(r#"{{"formato":{formato},"atualizado":"2026-11-05","traducoes":[{}]}}"#, fichas.join(","))
}

const URL: &str = "https://github.com/x/y/releases/download/v1/p.xdelta";

#[test]
fn le_indice_valido() {
    let i = ler_indice(&indice(1, &[ficha("a", URL, SHA, true), ficha("b", URL, SHA, true)]), false).unwrap();
    assert_eq!(i.traducoes.len(), 2);
    assert_eq!(i.atualizado, "2026-11-05");
    let d = i.traducoes[0].download.as_ref().unwrap();
    assert_eq!(d.patch.url, URL);
    assert_eq!(d.patch.tamanho, 238940);
    assert!(d.capa.is_some() && d.fundo.is_none());
}

#[test]
fn ficha_sem_download_no_indice_e_recusada() {
    assert!(matches!(ler_indice(&indice(1, &[ficha("a", URL, SHA, false)]), false), Err(Erro::IndiceInvalido(_))));
}

#[test]
fn url_http_e_recusada_sem_permissao() {
    let f = ficha("a", "http://exemplo.org/p.xdelta", SHA, true);
    assert!(matches!(ler_indice(&indice(1, &[f]), false), Err(Erro::IndiceInvalido(_))));
}

#[test]
fn http_local_e_aceito_com_permissao() {
    let f = ficha("a", "http://127.0.0.1:5000/p.xdelta", SHA, true);
    assert!(ler_indice(&indice(1, &[f.clone()]), true).is_ok());
    assert!(ler_indice(&indice(1, &[f]), false).is_err());
    let externo = ficha("b", "http://exemplo.org/p.xdelta", SHA, true);
    assert!(ler_indice(&indice(1, &[externo]), true).is_err());
}

#[test]
fn sha256_mal_formado_recusa() {
    assert!(ler_indice(&indice(1, &[ficha("a", URL, "abc", true)]), false).is_err());
}

#[test]
fn id_repetido_recusa() {
    assert!(ler_indice(&indice(1, &[ficha("a", URL, SHA, true), ficha("a", URL, SHA, true)]), false).is_err());
}

#[test]
fn formato_2_recusa() {
    assert!(ler_indice(&indice(2, &[ficha("a", URL, SHA, true)]), false).is_err());
}

#[test]
fn json_quebrado_recusa() {
    assert!(matches!(ler_indice("{ quebrado", false), Err(Erro::IndiceInvalido(_))));
}

#[test]
#[ignore = "precisa do indice.json do catálogo em TRADUMIX_INDICE"]
fn indice_real_do_catalogo_e_valido() {
    let caminho = std::env::var("TRADUMIX_INDICE").expect("defina TRADUMIX_INDICE");
    let i = ler_indice(&std::fs::read_to_string(caminho).unwrap(), false).unwrap();
    assert!(i.traducoes.iter().any(|f| f.id == "jak2-usa-ptbr"));
}
