use nucleo::{Erro, ler_indice};

const HASH: &str = "f51c94dd47cc2eb385d5733389428121";
const SHA: &str = "3ce44ebfecc30fc110c57f4fdf6ec0cd8c483fb3403d60ef3a671afb5f26e8de";

mod comum;

pub fn ficha(id: &str, url: &str, sha: &str, com_capa: bool) -> String {
    let imagens = if com_capa {
        format!(r#","imagens":{{"capa":{{"nome":"capa.jpg","url":"{url}.jpg","sha256":"{sha}","tamanho":10}}}}"#)
    } else {
        String::new()
    };
    comum::ficha_ps2(id, "1.0", HASH, 4_410_671_104, 4_410_671_104, url, sha, 238_940, &imagens)
}

fn indice(formato: u32, fichas: &[String]) -> String {
    format!(r#"{{"formato":{formato},"atualizado":"2026-11-05","traducoes":[{}]}}"#, fichas.join(","))
}

const URL: &str = "https://github.com/x/y/releases/download/v1/p.xdelta";

fn ids(i: &nucleo::Indice) -> Vec<&str> {
    i.traducoes.iter().map(|f| f.id.as_str()).collect()
}

#[test]
fn le_indice_valido() {
    let i = ler_indice(&indice(2, &[ficha("a", URL, SHA, true), ficha("b", URL, SHA, true)]), false).unwrap();
    assert_eq!(ids(&i), vec!["a", "b"]);
    assert_eq!(i.atualizado, "2026-11-05");
    assert!(i.ignoradas.is_empty());
    let f = &i.traducoes[0];
    assert_eq!(f.patch().url, URL);
    assert_eq!(f.patch().tamanho, 238940);
    let imagens = f.imagens.as_ref().unwrap();
    assert!(imagens.capa.is_some() && imagens.fundo.is_none());
}

#[test]
fn formato_1_recusa() {
    assert!(matches!(ler_indice(&indice(1, &[ficha("a", URL, SHA, true)]), false), Err(Erro::IndiceInvalido(_))));
}

#[test]
fn json_quebrado_recusa() {
    assert!(matches!(ler_indice("{ quebrado", false), Err(Erro::IndiceInvalido(_))));
}

#[test]
fn sem_traducoes_recusa() {
    assert!(matches!(ler_indice(r#"{"formato":2,"atualizado":"2026-11-05"}"#, false), Err(Erro::IndiceInvalido(_))));
}

#[test]
fn plataforma_desconhecida_fica_de_fora() {
    let pc = ficha("b", URL, SHA, true).replace(r#""plataforma":"ps2""#, r#""plataforma":"pc""#);
    let i = ler_indice(&indice(2, &[ficha("a", URL, SHA, true), pc, ficha("c", URL, SHA, true)]), false).unwrap();
    assert_eq!(ids(&i), vec!["a", "c"]);
    assert_eq!(i.ignoradas.len(), 1);
}

#[test]
fn instalacao_desconhecida_fica_de_fora() {
    let outra = ficha("b", URL, SHA, true).replace("imagem-xdelta", "pasta-de-arquivos");
    let i = ler_indice(&indice(2, &[ficha("a", URL, SHA, true), outra]), false).unwrap();
    assert_eq!(ids(&i), vec!["a"]);
}

#[test]
fn ficha_que_nao_e_objeto_fica_de_fora() {
    let i = ler_indice(&indice(2, &[ficha("a", URL, SHA, true), "42".into()]), false).unwrap();
    assert_eq!(ids(&i), vec!["a"]);
    assert_eq!(i.ignoradas.len(), 1);
}

#[test]
fn url_http_fica_de_fora_sem_permissao() {
    let f = ficha("a", "http://exemplo.org/p.xdelta", SHA, true);
    assert!(ler_indice(&indice(2, &[f]), false).unwrap().traducoes.is_empty());
}

#[test]
fn http_local_e_aceito_com_permissao() {
    let f = ficha("a", "http://127.0.0.1:5000/p.xdelta", SHA, true);
    assert_eq!(ler_indice(&indice(2, &[f.clone()]), true).unwrap().traducoes.len(), 1);
    assert!(ler_indice(&indice(2, &[f]), false).unwrap().traducoes.is_empty());
    let externo = ficha("b", "http://exemplo.org/p.xdelta", SHA, true);
    assert!(ler_indice(&indice(2, &[externo]), true).unwrap().traducoes.is_empty());
}

#[test]
fn sha256_mal_formado_fica_de_fora() {
    assert!(ler_indice(&indice(2, &[ficha("a", URL, "abc", true)]), false).unwrap().traducoes.is_empty());
}

#[test]
fn id_repetido_vale_a_primeira() {
    let nova = ficha("a", URL, SHA, true).replace(r#""versao":"1.0""#, r#""versao":"2.0""#);
    let i = ler_indice(&indice(2, &[ficha("a", URL, SHA, true), nova]), false).unwrap();
    assert_eq!(i.traducoes.len(), 1);
    assert_eq!(i.traducoes[0].versao, "1.0");
}

#[test]
#[ignore = "precisa do indice-v2.json do catálogo em TRADUMIX_INDICE"]
fn indice_real_do_catalogo_e_valido() {
    let caminho = std::env::var("TRADUMIX_INDICE").expect("defina TRADUMIX_INDICE");
    let i = ler_indice(&std::fs::read_to_string(caminho).unwrap(), false).unwrap();
    assert!(i.ignoradas.is_empty(), "{:?}", i.ignoradas);
    assert!(i.traducoes.iter().any(|f| f.id == "jak2-usa-ptbr"));
}

#[test]
fn trailer_com_id_invalido_fica_de_fora() {
    let com = |id: &str| {
        let mut f = ficha("a", URL, SHA, true);
        f.pop();
        format!(r#"{f},"midia":{{"trailer_youtube":"{id}"}}}}"#)
    };
    assert_eq!(ler_indice(&indice(2, &[com("Pj-ueiUYEbg")]), false).unwrap().traducoes.len(), 1);
    assert!(ler_indice(&indice(2, &[com("https://youtube.com/watch?v=Pj-ueiUYEbg")]), false).unwrap().traducoes.is_empty());
    assert!(ler_indice(&indice(2, &[com("curto")]), false).unwrap().traducoes.is_empty());
}
