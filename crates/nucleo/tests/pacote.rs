mod comum;

use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use nucleo::{Estado, Imagem, carregar_pacotes, comparar_versoes};

pub const HASH: &str = "f51c94dd47cc2eb385d5733389428121";
const SHA: &str = "3ce44ebfecc30fc110c57f4fdf6ec0cd8c483fb3403d60ef3a671afb5f26e8de";

/// Ficha formato 2 com textos e capa própria (`capa.jpg`).
fn ficha_json(id: &str, hash: &str) -> String {
    let extra = format!(
        r#","permissao":null,"descricao":"Tradução completa.","traduzido":["Menus","Legendas das cenas"],"limitacoes":["Texturas em inglês."],
        "imagens":{{"capa":{{"nome":"capa.jpg","url":"https://x/capa.jpg","sha256":"{SHA}","tamanho":10}}}}"#
    );
    comum::ficha_ps2(id, "1.0", hash, 4_410_671_104, 4_410_671_104, "https://x/p.xdelta", SHA, 238_940, &extra)
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
    criar_pacote(raiz.path(), "jak2", &ficha_json("jak2-usa-ptbr", HASH), true);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.invalidos.is_empty(), "{:?}", carga.invalidos.iter().map(|i| &i.motivo).collect::<Vec<_>>());
    assert_eq!(carga.pacotes.len(), 1);
    let p = &carga.pacotes[0];
    assert_eq!(p.ficha.id, "jak2-usa-ptbr");
    assert_eq!(p.ficha.estado, Estado::Beta);
    assert_eq!(p.ficha.plataforma, "ps2");
    assert_eq!(p.alvo.detalhes(), "versão USA (SCUS-97265)");
    assert_eq!(p.alvo.tamanho_resultado(), 4_410_671_104);
    assert_eq!(p.patch(), raiz.path().join("jak2").join("patch.xdelta"));
    assert!(p.patch_disponivel());
    assert_eq!(p.imagem(Imagem::Capa), Some(raiz.path().join("jak2").join("capa.jpg")));
    assert_eq!(p.imagem(Imagem::Fundo), None);
}

#[test]
fn pasta_com_ficha_quebrada_vira_invalida_e_as_outras_carregam() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "bom", &ficha_json("bom", HASH), true);
    criar_pacote(raiz.path(), "quebrado", "{ isto não é json", true);
    let carga = carregar_pacotes(raiz.path());
    assert_eq!(carga.pacotes.len(), 1);
    assert_eq!(carga.invalidos.len(), 1);
    assert_eq!(carga.invalidos[0].pasta, raiz.path().join("quebrado"));
    assert!(!carga.invalidos[0].motivo.is_empty());
}

#[test]
fn hash_ra_mal_formado_e_invalido() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "hash", &ficha_json("x", "1234"), true);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.pacotes.is_empty());
    assert_eq!(carga.invalidos.len(), 1);
}

#[test]
fn formato_1_e_invalido() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "f1", &ficha_json("x", HASH).replace(r#""formato":2"#, r#""formato":1"#), true);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.pacotes.is_empty());
    assert_eq!(carga.invalidos.len(), 1);
}

#[test]
fn instalacao_desconhecida_e_invalida() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "i", &ficha_json("x", HASH).replace("imagem-xdelta", "pasta-de-arquivos"), true);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.pacotes.is_empty());
    assert_eq!(carga.invalidos.len(), 1);
}

#[test]
fn plataforma_desconhecida_e_invalida() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "p", &ficha_json("x", HASH).replace(r#""plataforma":"ps2""#, r#""plataforma":"wii""#), true);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.pacotes.is_empty());
    assert!(carga.invalidos[0].motivo.contains("plataforma desconhecida"));
}

#[test]
fn arquivo_fora_da_pasta_do_pacote_e_invalido() {
    let raiz = tempfile::tempdir().unwrap();
    let json = ficha_json("x", HASH).replace(r#""nome":"patch.xdelta""#, r#""nome":"../fora.xdelta""#);
    criar_pacote(raiz.path(), "fuga", &json, false);
    let json_capa = ficha_json("y", HASH).replace(r#""nome":"capa.jpg""#, r#""nome":"C:\\Windows\\win.ini""#);
    criar_pacote(raiz.path(), "fuga-capa", &json_capa, true);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.pacotes.is_empty());
    assert_eq!(carga.invalidos.len(), 2);
}

#[test]
fn patch_chamado_ficha_json_e_invalido() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "f", &ficha_json("x", HASH).replace(r#""nome":"patch.xdelta""#, r#""nome":"ficha.json""#), false);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.pacotes.is_empty());
    assert_eq!(carga.invalidos.len(), 1);
}

#[test]
fn capa_do_acervo_na_pasta_vira_a_capa() {
    let raiz = tempfile::tempdir().unwrap();
    let json = comum::ficha_catalogo("x", "1.0", "https://x", HASH, 100, b"P", None);
    criar_pacote(raiz.path(), "sem-capa", &json, false);
    let p = carregar_pacotes(raiz.path()).pacotes.remove(0);
    assert_eq!(p.imagem(Imagem::Capa), None);
    fs::write(raiz.path().join("sem-capa").join("capa.jpg"), b"JPG").unwrap();
    assert_eq!(p.imagem(Imagem::Capa), Some(raiz.path().join("sem-capa").join("capa.jpg")));
}

#[test]
fn acha_pacotes_em_subpastas() {
    let raiz = tempfile::tempdir().unwrap();
    let fundo = raiz.path().join("Jak II").join("pacote");
    criar_pacote(&fundo, "jak2-usa-ptbr", &ficha_json("jak2-usa-ptbr", HASH), true);
    fs::create_dir_all(raiz.path().join("Jak II").join("tools")).unwrap();
    let carga = carregar_pacotes(raiz.path());
    assert_eq!(carga.pacotes.len(), 1, "{:?}", carga.invalidos);
    assert_eq!(carga.pacotes[0].pasta, fundo.join("jak2-usa-ptbr"));
}

#[test]
fn nao_desce_dentro_de_um_pacote() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "a", &ficha_json("a", HASH), true);
    criar_pacote(&raiz.path().join("a"), "dentro", &ficha_json("dentro", HASH), true);
    let carga = carregar_pacotes(raiz.path());
    assert_eq!(carga.pacotes.len(), 1);
    assert_eq!(carga.pacotes[0].ficha.id, "a");
}

#[test]
fn pacote_carrega_sem_o_patch_baixado() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "cat", &ficha_json("cat", HASH), false);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.invalidos.is_empty(), "{:?}", carga.invalidos);
    assert!(!carga.pacotes[0].patch_disponivel());
}

fn ficha_de(data: &str) -> nucleo::Ficha {
    let raiz = tempfile::tempdir().unwrap();
    let json = ficha_json("x", HASH).replace("2026-11-05", data);
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
fn versoes() {
    assert_eq!(comparar_versoes("1.0", "1.1"), Ordering::Less);
    assert_eq!(comparar_versoes("1.9", "1.10"), Ordering::Less);
    assert_eq!(comparar_versoes("1.0", "1.0"), Ordering::Equal);
    assert_eq!(comparar_versoes("2", "1.9"), Ordering::Greater);
}

fn motivo_de(ficha: &str) -> String {
    let i = nucleo::ler_indice(std::str::from_utf8(&comum::indice(&[ficha.to_string()])).unwrap(), false).unwrap();
    assert!(i.traducoes.is_empty(), "a ficha devia ficar de fora");
    i.ignoradas.join(" | ")
}

#[test]
fn ficha_pc_lida_sem_programa() {
    let raiz = tempfile::tempdir().unwrap();
    criar_pacote(raiz.path(), "pc", &comum::ficha_pc_simples(&["mods/textos.sub", "dados/menu.txt"]), false);
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.invalidos.is_empty(), "{:?}", carga.invalidos);
    let f = &carga.pacotes[0].ficha;
    assert!(!f.contem_programa());
    assert_eq!(f.baixavel_principal().nome, "pacote.zip");
    assert_eq!(carga.pacotes[0].patch(), raiz.path().join("pc").join("pacote.zip"));
}

#[test]
fn ficha_pc_com_dll() {
    let ficha: nucleo::Ficha = serde_json::from_str(&comum::ficha_pc_simples(&["mods/carregador.DLL"])).unwrap();
    assert!(ficha.contem_programa());
    for ext in ["exe", "bat", "cmd", "ps1", "vbs", "js", "py", "msi", "scr", "com"] {
        let f: nucleo::Ficha = serde_json::from_str(&comum::ficha_pc_simples(&[&format!("a/b.{ext}")])).unwrap();
        assert!(f.contem_programa(), "{ext}");
    }
    let ps2: nucleo::Ficha = serde_json::from_str(&ficha_json("a", HASH)).unwrap();
    assert!(!ps2.contem_programa());
}

#[test]
fn ficha_pc_caminho_ruim() {
    for ruim in ["../x.txt", "a/../../x", "C:/x", "/x", r"\\servidor\x", r"a\b.txt","a//b", "CON.txt", "a/nul", "x?.txt", "a/com1", "lpt9.log", "a/b:c", "a/.", ""] {
        let motivo = motivo_de(&comum::ficha_pc_simples(&[ruim]));
        assert!(motivo.contains("caminho"), "{ruim}: {motivo}");
    }
}

#[test]
fn ficha_pc_referencia_com_caminho_ruim() {
    let ficha = comum::ficha_pc_simples(&["a.txt"]).replace(r#""caminho":"jogo.exe""#, r#""caminho":"../jogo.exe""#);
    assert!(motivo_de(&ficha).contains("caminho"));
}

#[test]
fn ficha_pc_aceita_nomes_parecidos_com_reservados() {
    let ficha: nucleo::Ficha = serde_json::from_str(&comum::ficha_pc_simples(&["console.txt", "a/com10.txt", "nulo/auxiliar.dat"])).unwrap();
    assert!(nucleo::ler_alvo(&ficha).is_ok());
}

#[test]
fn ficha_pc_sem_referencias() {
    let ficha = comum::ficha_pc("x", "1.0", comum::SHA_A, 1, &format!(r#"[{{"caminho":"a.txt","sha256":"{}","tamanho":1}}]"#, comum::SHA_A), "[]");
    assert!(motivo_de(&ficha).contains("referencias"));
}

#[test]
fn ficha_pc_sha_ruim() {
    let ficha = comum::ficha_pc("x", "1.0", comum::SHA_A, 1, r#"[{"caminho":"a.txt","sha256":"123","tamanho":1}]"#, &format!(r#"[{{"caminho":"j.exe","sha256":"{}"}}]"#, comum::SHA_A));
    assert!(motivo_de(&ficha).contains("sha256"));
}

#[test]
fn ficha_ps2_com_pasta_do_jogo() {
    let ficha = comum::ficha_pc_simples(&["a.txt"])
        .replace(r#""plataforma":"pc""#, r#""plataforma":"ps2""#)
        .replace(r#""origem":{"loja":"Steam","versao":"1.0.3","referencias":"#, r#""origem_pc":{"x":"#)
        .replace(r#""resultado":{}"#, &format!(r#""origem":{{"serial":"SCUS-97265","regiao":"usa","hash_ra":"{HASH}","tamanho":1}},"resultado":{{"hash_ra":"{HASH}","tamanho":1}}"#));
    assert!(motivo_de(&ficha).contains("tipo de instalação não combina com a plataforma"), "{}", motivo_de(&ficha));
}

#[test]
fn ficha_pc_com_imagem_xdelta() {
    let ficha = comum::ficha_pc_simples(&["a.txt"]).replace(
        r#""tipo":"pasta-do-jogo","pacote":"#,
        r#""tipo":"imagem-xdelta","patch":"#,
    );
    assert!(motivo_de(&ficha).contains("tipo de instalação não combina com a plataforma"), "{}", motivo_de(&ficha));
}

#[test]
fn ficha_ps2_antiga_igual() {
    let ficha: nucleo::Ficha = serde_json::from_str(&ficha_json("jak2", HASH)).unwrap();
    assert_eq!(ficha.baixavel_principal().nome, "patch.xdelta");
    assert_eq!(ficha.baixavel_principal().tamanho, 238_940);
}

#[test]
fn ficha_pc_nao_grava_no_registro_da_tradumix() {
    for ruim in [".tradumix/jogo-pc-ptbr/originais/a.txt", ".Tradumix/x", ".TRADUMIX"] {
        assert!(motivo_de(&comum::ficha_pc_simples(&[ruim])).contains("caminho"), "{ruim}");
    }
}

#[test]
fn ficha_pc_com_caminhos_que_so_diferem_na_caixa() {
    let motivo = motivo_de(&comum::ficha_pc_simples(&["a.txt", "A.TXT"]));
    assert!(motivo.contains("repetido"), "{motivo}");
}
