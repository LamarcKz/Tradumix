mod comum;

use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use nucleo::{Estado, IdentidadePs2, Regiao, agrupar_por_jogo, carregar_pacotes, comparar_versoes, serve_para};

pub const HASH: &str = "f51c94dd47cc2eb385d5733389428121";

fn ficha_json(id: &str, jogo_id: &str, nome: &str, hash: &str, formato: u32) -> String {
    format!(
        r#"{{
  "formato": {formato},
  "id": "{id}",
  "jogo": {{ "id": "{jogo_id}", "nome": "{nome}", "console": "ps2" }},
  "versao": "1.0",
  "estado": "rascunho",
  "idioma": "pt-BR",
  "data": "2026-11-05",
  "origem": {{ "serial": "SCUS-97265", "regiao": "usa", "hash_ra": "{hash}", "tamanho": 4410671104 }},
  "resultado": {{ "hash_ra": "{hash}", "tamanho": 4410671104 }},
  "tradutores": [ {{ "nome": "Fulano", "link": null }} ],
  "permissao": null,
  "descricao": "Tradução completa.",
  "traduzido": ["Menus", "Legendas das cenas"],
  "limitacoes": ["Texturas em inglês."],
  "arquivos": {{ "patch": "patch.xdelta", "capa": "capa.jpg" }}
}}"#
    )
}

fn criar_pacote(raiz: &Path, pasta: &str, json: &str, com_patch: bool) {
    let p = raiz.join(pasta);
    fs::create_dir_all(&p).unwrap();
    fs::write(p.join("ficha.json"), json).unwrap();
    if com_patch {
        fs::write(p.join("patch.xdelta"), b"VCD").unwrap();
    }
}

#[test]
fn le_ficha_valida() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "jak2", &ficha_json("jak2-usa-ptbr", "jak2", "Jak II", HASH, 1), true);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.invalidos.is_empty(), "{:?}", carga.invalidos.iter().map(|i| &i.motivo).collect::<Vec<_>>());
    assert_eq!(carga.pacotes.len(), 1);
    let p = &carga.pacotes[0];
    assert_eq!(p.ficha.id, "jak2-usa-ptbr");
    assert_eq!(p.ficha.estado, Estado::Rascunho);
    assert_eq!(p.ficha.origem.regiao, Regiao::Usa);
    assert_eq!(p.ficha.resultado.tamanho, 4_410_671_104);
    assert_eq!(p.patch(), raiz.path().join("jak2").join("patch.xdelta"));
    assert_eq!(p.imagem(nucleo::Imagem::Capa), Some(raiz.path().join("jak2").join("capa.jpg")));
    assert_eq!(p.imagem(nucleo::Imagem::Fundo), None);
}

#[test]
fn pasta_com_ficha_quebrada_vira_invalida_e_as_outras_carregam() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "bom", &ficha_json("bom", "jak2", "Jak II", HASH, 1), true);
    criar_pacote(raiz.path(), "quebrado", "{ isto não é json", true);
    let carga = carregar_pacotes(raiz.path());
    assert_eq!(carga.pacotes.len(), 1);
    assert_eq!(carga.invalidos.len(), 1);
    assert_eq!(carga.invalidos[0].pasta, raiz.path().join("quebrado"));
    assert!(!carga.invalidos[0].motivo.is_empty());
}

#[test]
fn patch_ausente_e_invalido() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "sem-patch", &ficha_json("x", "jak2", "Jak II", HASH, 1), false);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.pacotes.is_empty());
    assert!(carga.invalidos[0].motivo.contains("patch.xdelta"));
}

#[test]
fn hash_ra_mal_formado_e_invalido() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "hash", &ficha_json("x", "jak2", "Jak II", "1234", 1), true);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.pacotes.is_empty());
    assert_eq!(carga.invalidos.len(), 1);
}

#[test]
fn formato_desconhecido_e_invalido() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "f2", &ficha_json("x", "jak2", "Jak II", HASH, 2), true);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.pacotes.is_empty());
    assert_eq!(carga.invalidos.len(), 1);
}

#[test]
fn arquivo_fora_da_pasta_do_pacote_e_invalido() {
    let raiz = tempfile::tempdir().unwrap();
    fs::write(raiz.path().join("fora.xdelta"), b"VCD").unwrap();
    let json = ficha_json("x", "jak2", "Jak II", HASH, 1).replace("\"patch.xdelta\"", "\"../fora.xdelta\"");
    criar_pacote(raiz.path(), "fuga", &json, false);
    let json_capa = ficha_json("y", "jak2", "Jak II", HASH, 1).replace("\"capa.jpg\"", "\"C:\\\\Windows\\\\win.ini\"");
    criar_pacote(raiz.path(), "fuga-capa", &json_capa, true);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.pacotes.is_empty());
    assert_eq!(carga.invalidos.len(), 2);
}

/// Escolher a pasta da oficina (`Traduções`) acha o pacote em `Jak II\pacote\jak2-usa-ptbr`.
#[test]
fn acha_pacotes_em_subpastas() {
    let raiz = tempfile::tempdir().unwrap();
    let fundo = raiz.path().join("Jak II").join("pacote");
    criar_pacote(&fundo, "jak2-usa-ptbr", &ficha_json("jak2-usa-ptbr", "jak2", "Jak II", HASH, 1), true);
    fs::create_dir_all(raiz.path().join("Jak II").join("tools")).unwrap();
    let carga = carregar_pacotes(raiz.path());
    assert_eq!(carga.pacotes.len(), 1, "{:?}", carga.invalidos);
    assert_eq!(carga.pacotes[0].pasta, fundo.join("jak2-usa-ptbr"));
}

#[test]
fn nao_desce_dentro_de_um_pacote() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "a", &ficha_json("a", "jak2", "Jak II", HASH, 1), true);
    criar_pacote(&raiz.path().join("a"), "dentro", &ficha_json("dentro", "jak2", "Jak II", HASH, 1), true);
    let carga = carregar_pacotes(raiz.path());
    assert_eq!(carga.pacotes.len(), 1);
    assert_eq!(carga.pacotes[0].ficha.id, "a");
}

#[test]
fn pacote_com_download_carrega_sem_o_patch() {
    let raiz = tempfile::tempdir().unwrap();
    let sha = "3ce44ebfecc30fc110c57f4fdf6ec0cd8c483fb3403d60ef3a671afb5f26e8de";
    let json = ficha_json("cat", "jak2", "Jak II", HASH, 1).replace(
        "\"arquivos\":",
        &format!(r#""download": {{ "patch": {{ "url": "https://x/p.xdelta", "sha256": "{sha}", "tamanho": 10 }} }}, "arquivos":"#),
    );
    criar_pacote(raiz.path(), "cat", &json, false);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.invalidos.is_empty(), "{:?}", carga.invalidos);
    assert!(!carga.pacotes[0].patch_disponivel());
}

fn ficha_de(data: &str) -> nucleo::Ficha {
    let raiz = tempfile::tempdir().unwrap();
    let json = ficha_json("x", "jak2", "Jak II", HASH, 1).replace("2026-11-05", data);
    criar_pacote(raiz.path(), "x", &json, true);
    carregar_pacotes(raiz.path()).pacotes.remove(0).ficha
}

#[test]
fn novo_no_mesmo_dia() {
    assert!(nucleo::e_novo(&ficha_de("2026-11-05"), "2026-11-05"));
}

#[test]
fn novo_ate_13_dias() {
    assert!(nucleo::e_novo(&ficha_de("2026-11-05"), "2026-11-18"));
}

#[test]
fn nao_e_novo_com_14_dias() {
    assert!(!nucleo::e_novo(&ficha_de("2026-11-05"), "2026-11-19"));
}

#[test]
fn data_no_futuro_nao_e_novo() {
    assert!(!nucleo::e_novo(&ficha_de("2026-11-10"), "2026-11-05"));
}

#[test]
fn data_invalida_nao_e_novo() {
    assert!(!nucleo::e_novo(&ficha_de("ontem"), "2026-11-05"));
}

#[test]
fn pasta_inexistente_nao_quebra() {
    let carga = carregar_pacotes(Path::new("pasta-que-nao-existe-xyz"));
    assert!(carga.pacotes.is_empty());
}

#[test]
fn agrupa_duas_traducoes_do_mesmo_jogo() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "a", &ficha_json("jak2-a", "jak2", "Jak II", HASH, 1), true);
    criar_pacote(raiz.path(), "b", &ficha_json("jak2-b", "jak2", "Jak II", HASH, 1), true);
    criar_pacote(raiz.path(), "c", &ficha_json("abc-1", "abc", "Abc", HASH, 1), true);
    let carga = carregar_pacotes(raiz.path());
    let jogos = agrupar_por_jogo(&carga.pacotes);
    assert_eq!(jogos.len(), 2);
    assert_eq!(jogos[0].nome, "Abc");
    assert_eq!(jogos[1].id, "jak2");
    let mut ids = jogos[1].pacotes.clone();
    ids.sort();
    assert_eq!(ids, vec!["jak2-a", "jak2-b"]);
}

#[test]
fn serve_para_compara_hash_de_origem() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "a", &ficha_json("jak2-a", "jak2", "Jak II", HASH, 1), true);
    let carga = carregar_pacotes(raiz.path());
    let mut id = IdentidadePs2 {
        serial: "SCUS-97265".into(),
        executavel: "SCUS_972.65".into(),
        regiao: Regiao::Usa,
        hash_ra: HASH.into(),
        tamanho: 4_410_671_104,
    };
    assert!(serve_para(&carga.pacotes[0].ficha, &id));
    id.hash_ra = "00000000000000000000000000000000".into();
    assert!(!serve_para(&carga.pacotes[0].ficha, &id));
}

#[test]
fn versoes() {
    assert_eq!(comparar_versoes("1.0", "1.1"), Ordering::Less);
    assert_eq!(comparar_versoes("1.9", "1.10"), Ordering::Less);
    assert_eq!(comparar_versoes("1.0", "1.0"), Ordering::Equal);
    assert_eq!(comparar_versoes("2", "1.9"), Ordering::Greater);
}
