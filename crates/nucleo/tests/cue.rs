mod comum;

use std::fs;

use nucleo::{Erro, TipoFaixa, escrever_cue, ler_cue};

fn cue(texto: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let pasta = tempfile::tempdir().unwrap();
    let caminho = pasta.path().join("Jogo (USA).cue");
    fs::write(&caminho, texto).unwrap();
    (pasta, caminho)
}

#[test]
fn cue_com_uma_faixa() {
    let (pasta, caminho) = cue("FILE \"Jogo (USA).bin\" BINARY\r\n  TRACK 01 MODE2/2352\r\n    INDEX 01 00:00:00\r\n");
    let c = ler_cue(&caminho).unwrap();
    assert_eq!(c.faixas.len(), 1);
    assert_eq!(c.faixas[0].numero, 1);
    assert_eq!(c.faixas[0].tipo, TipoFaixa::Mode2_2352);
    assert_eq!(c.faixas[0].arquivo, pasta.path().join("Jogo (USA).bin"));
}

#[test]
fn cue_com_faixas_de_audio() {
    let (_p, caminho) = cue(
        "FILE \"J (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n\
         FILE \"J (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:02:00\n\
         FILE \"J (Track 3).bin\" BINARY\n  TRACK 03 AUDIO\n    INDEX 01 00:00:00\n",
    );
    let c = ler_cue(&caminho).unwrap();
    assert_eq!(c.faixas.len(), 3);
    assert_eq!(c.faixas[1].tipo, TipoFaixa::Audio);
    assert_eq!(c.faixa_de_dados().unwrap().numero, 1);
}

#[test]
fn cue_com_acento_e_subpasta() {
    let (pasta, caminho) = cue("FILE \"faixas/Ação 1.bin\" BINARY\n  TRACK 01 MODE1/2352\n");
    let c = ler_cue(&caminho).unwrap();
    assert_eq!(c.faixas[0].tipo, TipoFaixa::Mode1_2352);
    assert_eq!(c.faixas[0].arquivo, pasta.path().join("faixas").join("Ação 1.bin"));
}

#[test]
fn cue_em_latin1() {
    let pasta = tempfile::tempdir().unwrap();
    let caminho = pasta.path().join("a.cue");
    let mut bytes = b"FILE \"A".to_vec();
    bytes.push(0xE7); // ç em Latin-1
    bytes.extend(b"o.bin\" BINARY\n  TRACK 01 MODE2/2352\n");
    fs::write(&caminho, bytes).unwrap();
    assert_eq!(ler_cue(&caminho).unwrap().faixas[0].arquivo, pasta.path().join("Aço.bin"));
}

#[test]
fn cue_sem_aspas() {
    let (pasta, caminho) = cue("FILE jogo.bin BINARY\n  TRACK 01 MODE2/2352\n");
    assert_eq!(ler_cue(&caminho).unwrap().faixas[0].arquivo, pasta.path().join("jogo.bin"));
}

#[test]
fn cue_sem_faixa_de_dados() {
    let (_p, caminho) = cue("FILE \"a.bin\" BINARY\n  TRACK 01 AUDIO\n");
    assert!(ler_cue(&caminho).unwrap().faixa_de_dados().is_none());
}

#[test]
fn cue_inexistente() {
    let pasta = tempfile::tempdir().unwrap();
    let erro = ler_cue(&pasta.path().join("nada.cue")).unwrap_err();
    assert!(matches!(erro, Erro::CueInvalido(_)));
    assert!(erro.mensagem().starts_with("O arquivo .cue não pôde ser lido: "));
}

#[test]
fn escrever_cue_troca_so_os_nomes() {
    let original = "FILE \"Velho 1.bin\" BINARY\r\n  TRACK 01 MODE2/2352\r\n    INDEX 01 00:00:00\r\n\
                    FILE \"Velho 2.bin\" BINARY\r\n  TRACK 02 AUDIO\r\n    INDEX 00 00:00:00\r\n    INDEX 01 00:02:00\r\n";
    let (_p, caminho) = cue(original);
    let c = ler_cue(&caminho).unwrap();
    let novo = escrever_cue(&c, &["Novo 1.bin".into(), "Novo 2.bin".into()]);
    assert_eq!(novo, original.replace("Velho", "Novo"));
}

#[test]
fn cue_com_file_em_outra_pasta_e_barra_invertida() {
    let pasta = tempfile::tempdir().unwrap();
    fs::create_dir(pasta.path().join("cues")).unwrap();
    let caminho = pasta.path().join("cues").join("j.cue");
    fs::write(&caminho, "FILE \"..\\bins\\Jogo (Disco 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n").unwrap();
    let arquivo = &ler_cue(&caminho).unwrap().faixas[0].arquivo;
    fs::create_dir(pasta.path().join("bins")).unwrap();
    fs::write(pasta.path().join("bins").join("Jogo (Disco 1).bin"), b"x").unwrap();
    assert_eq!(fs::read(arquivo).unwrap(), b"x");
}
