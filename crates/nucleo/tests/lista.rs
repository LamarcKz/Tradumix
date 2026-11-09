mod comum;

use std::fs;
use std::path::Path;

use nucleo::lista::{SituacaoRa, montar_jogos};
use nucleo::{Pacote, carregar_pacotes};

const OUTRO_HASH: &str = "0123456789abcdef0123456789abcdef";

fn pacotes(fichas: &[(&str, String)]) -> (tempfile::TempDir, Vec<Pacote>) {
    let raiz = tempfile::tempdir().unwrap();
    for (pasta, ficha) in fichas {
        let p = raiz.path().join(pasta);
        fs::create_dir_all(&p).unwrap();
        fs::write(p.join("ficha.json"), ficha).unwrap();
    }
    let carga = carregar_pacotes(raiz.path());
    assert!(carga.invalidos.is_empty(), "{:?}", carga.invalidos);
    (raiz, carga.pacotes)
}

fn ps1(id: &str, resultado: &str) -> String {
    comum::ficha_ps1(id, comum::HASH_RA, resultado, 100, 100, comum::SHA_A, 10, "").replace(r#""id":"jogo-ps1""#, &format!(r#""id":"{id}-jogo""#))
}

#[test]
fn lista_marca_ra() {
    let (_r, p) = pacotes(&[
        ("a", ps1("a", comum::HASH_RA)),
        ("b", ps1("b", OUTRO_HASH)),
        ("c", comum::ficha_pc_simples(&["mods/textos.sub"])),
    ]);
    let jogos = montar_jogos(&p, "2026-11-09");
    let ra = |id: &str| {
        jogos.iter().flat_map(|j| &j.plataformas).flat_map(|p| &p.traducoes).find(|t| t.id == id).unwrap().ra
    };
    assert_eq!(ra("a"), Some(SituacaoRa::Compativel));
    assert_eq!(ra("b"), Some(SituacaoRa::NaoReconhece));
    assert_eq!(ra("jogo-pc-ptbr"), None);
}

#[test]
fn lista_contem_programa() {
    let (_r, p) = pacotes(&[("pc", comum::ficha_pc_simples(&["mods/carregador.dll"]))]);
    let jogos = montar_jogos(&p, "2026-11-09");
    let t = &jogos[0].plataformas[0].traducoes[0];
    assert!(t.contem_programa);
    assert_eq!(t.tamanho_download, Some(100));
    assert!(jogos[0].plataformas[0].info.escolhe_pasta);
}

#[test]
fn ra_e_selo_em_json() {
    let (_r, p) = pacotes(&[("b", ps1("b", OUTRO_HASH))]);
    let json = serde_json::to_value(&montar_jogos(&p, "2026-11-09")).unwrap();
    let t = &json[0]["plataformas"][0]["traducoes"][0];
    assert_eq!(t["ra"], "nao_reconhece");
    assert_eq!(t["contem_programa"], false);
}

#[test]
fn jogos_em_ordem_e_plataformas_na_ordem_central() {
    let ps2 = comum::ficha_ps2("jak2-ps2", "1.0", comum::HASH_RA, 1, 1, "https://x/p", comum::SHA_A, 1, "");
    let pc = comum::ficha_pc_simples(&["a.txt"]).replace(r#""id":"jogo-pc","nome":"Jogo de PC""#, r#""id":"jak2","nome":"Jak II""#);
    let ps1 = ps1("z", comum::HASH_RA).replace(r#""id":"z-jogo","nome":"Jogo de PS1""#, r#""id":"abc","nome":"Abc""#);
    let (_r, p) = pacotes(&[("1", pc), ("2", ps2), ("3", ps1)]);
    let jogos = montar_jogos(&p, "2026-11-09");
    assert_eq!(jogos.iter().map(|j| j.nome.as_str()).collect::<Vec<_>>(), ["Abc", "Jak II"]);
    let siglas: Vec<&str> = jogos[1].plataformas.iter().map(|p| p.info.sigla).collect();
    assert_eq!(siglas, ["PS2", "PC"]);
    let _ = Path::new("");
}
