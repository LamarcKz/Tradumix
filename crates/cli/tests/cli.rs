//! Roda o `tradumix-cli` de verdade sobre pacotes sintéticos.
#[path = "../../nucleo/tests/comum/mod.rs"]
mod comum;

use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

const CNF: &[u8] = b"BOOT = cdrom:\\SLUS_006.84;1\r\n";

fn cli(args: &[&str]) -> (i32, Value, String) {
    let saida: Output = Command::new(env!("CARGO_BIN_EXE_tradumix-cli")).args(args).output().unwrap();
    let texto = String::from_utf8_lossy(&saida.stdout).into_owned();
    let json = serde_json::from_str(&texto).unwrap_or_else(|e| panic!("saída não é JSON ({e}): {texto}"));
    (saida.status.code().unwrap(), json, String::from_utf8_lossy(&saida.stderr).into_owned())
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

fn xdelta3() -> PathBuf {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/binaries/xdelta3-x86_64-pc-windows-msvc.exe");
    assert!(p.is_file(), "xdelta3 não encontrado: rode npm run xdelta3");
    p
}

fn bin_ps1(semente: u8) -> Vec<u8> {
    let exe = comum::exe_psx(2048, 5);
    let dados: Vec<u8> = (0..30_000u32).map(|i| (i as u8).wrapping_mul(13).wrapping_add(semente)).collect();
    comum::bin_de_iso(&comum::iso_sintetica(&[("SYSTEM.CNF", CNF), ("SLUS_006.84", &exe), ("D.BIN", &dados)]), 24)
}

struct Ps1 {
    _t: tempfile::TempDir,
    jogo: PathBuf,
    cue: PathBuf,
    pacote: PathBuf,
    destino: Vec<u8>,
}

/// Disco PS1 (`.cue` + `.bin`) e um pacote com o patch gerado pelo xdelta3.
fn ps1() -> Ps1 {
    let t = tempfile::tempdir().unwrap();
    let jogo = t.path().join("jogo");
    fs::create_dir_all(&jogo).unwrap();
    let (origem, destino) = (bin_ps1(1), bin_ps1(2));
    comum::gravar(&jogo, "Jogo (USA).bin", &origem);
    let cue = comum::gravar(&jogo, "Jogo (USA).cue", b"FILE \"Jogo (USA).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
    let destino_tmp = comum::gravar(t.path(), "destino.tmp", &destino);
    let pacote = t.path().join("pacotes").join("jogo-ps1");
    fs::create_dir_all(&pacote).unwrap();
    let ok = Command::new(xdelta3())
        .args(["-e", "-9", "-S", "djw", "-f", "-s"])
        .arg(jogo.join("Jogo (USA).bin"))
        .arg(&destino_tmp)
        .arg(pacote.join("patch.xdelta"))
        .env_remove("XDELTA")
        .status()
        .unwrap();
    assert!(ok.success());
    let hash = nucleo::identificar_ps1(&cue).unwrap().hash_ra;
    let patch = fs::read(pacote.join("patch.xdelta")).unwrap();
    let ficha = comum::ficha_ps1("jogo-ps1-ptbr", &hash, &hash, origem.len() as u64, destino.len() as u64, &comum::sha256_hex(&patch), patch.len() as u64, "");
    fs::write(pacote.join("ficha.json"), ficha).unwrap();
    Ps1 { _t: t, jogo, cue, pacote, destino }
}

fn zip_de(entradas: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (nome, dados) in entradas {
        zip.start_file(*nome, zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(dados).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

/// Pacote de PC com `mods/carregador.dll` e uma pasta de jogo com `jogo.exe` e `dados/menu.txt`.
fn pc(raiz: &Path) -> (PathBuf, PathBuf) {
    let jogo = raiz.join("Jogo de PC");
    fs::create_dir_all(jogo.join("dados")).unwrap();
    fs::write(jogo.join("jogo.exe"), b"EXE").unwrap();
    fs::write(jogo.join("dados").join("menu.txt"), b"MENU EN").unwrap();
    let arquivos: &[(&str, &[u8])] = &[("dados/menu.txt", b"MENU PT"), ("mods/carregador.dll", b"DLL")];
    let zip = zip_de(arquivos);
    let lista: Vec<String> = arquivos
        .iter()
        .map(|(c, d)| format!(r#"{{"caminho":"{c}","sha256":"{}","tamanho":{}}}"#, comum::sha256_hex(d), d.len()))
        .collect();
    let refs = format!(r#"[{{"caminho":"jogo.exe","sha256":"{}"}}]"#, comum::sha256_hex(b"EXE"));
    let ficha = comum::ficha_pc("jogo-pc-ptbr", "1.0", &comum::sha256_hex(&zip), zip.len() as u64, &format!("[{}]", lista.join(",")), &refs);
    let pacote = raiz.join("pacotes").join("jogo-pc");
    fs::create_dir_all(&pacote).unwrap();
    fs::write(pacote.join("ficha.json"), ficha).unwrap();
    fs::write(pacote.join("pacote.zip"), zip).unwrap();
    (jogo, pacote)
}

#[test]
fn uso_errado() {
    let saida = Command::new(env!("CARGO_BIN_EXE_tradumix-cli")).output().unwrap();
    assert_eq!(saida.status.code(), Some(2));
    let (codigo, json, _) = cli(&["validar"]);
    assert_eq!(codigo, 2);
    assert_eq!(json["ok"], false);
    let (codigo, _, _) = cli(&["nao-existe", "x"]);
    assert_eq!(codigo, 2);
}

#[test]
fn validar_ok_ps1() {
    let c = ps1();
    let (codigo, json, _) = cli(&["validar", s(&c.pacote)]);
    assert_eq!(codigo, 0, "{json}");
    assert_eq!(json["ok"], true);
    assert_eq!(json["id"], "jogo-ps1-ptbr");
    assert_eq!(json["plataforma"], "ps1");
    assert_eq!(json["ra"], "compativel");
    assert_eq!(json["contem_programa"], false);
}

#[test]
fn validar_ficha_ruim() {
    let c = ps1();
    fs::write(c.pacote.join("ficha.json"), "{ quebrada").unwrap();
    let (codigo, json, _) = cli(&["validar", s(&c.pacote)]);
    assert_eq!(codigo, 1);
    assert_eq!(json["ok"], false);
    assert!(json["erro"].as_str().unwrap().contains("ficha.json"));
}

#[test]
fn validar_sha_errado() {
    let c = ps1();
    let mut patch = fs::read(c.pacote.join("patch.xdelta")).unwrap();
    let ultimo = patch.len() - 1;
    patch[ultimo] ^= 1;
    fs::write(c.pacote.join("patch.xdelta"), patch).unwrap();
    let (codigo, json, _) = cli(&["validar", s(&c.pacote)]);
    assert_eq!(codigo, 1);
    assert!(json["erro"].as_str().unwrap().contains("patch.xdelta"), "{json}");
}

#[test]
fn validar_pc_com_programa() {
    let t = tempfile::tempdir().unwrap();
    let (_, pacote) = pc(t.path());
    let (codigo, json, _) = cli(&["validar", s(&pacote)]);
    assert_eq!(codigo, 0, "{json}");
    assert_eq!(json["plataforma"], "pc");
    assert_eq!(json["ra"], Value::Null);
    assert_eq!(json["contem_programa"], true);
}

#[test]
fn identificar_ps1() {
    let c = ps1();
    let (codigo, json, _) = cli(&["identificar", s(&c.cue), "--plataforma", "ps1"]);
    assert_eq!(codigo, 0, "{json}");
    let id = nucleo::identificar_ps1(&c.cue).unwrap();
    assert_eq!(json["serial"], "SLUS-00684");
    assert_eq!(json["regiao"], "usa");
    assert_eq!(json["hash_ra"], id.hash_ra);
    assert_eq!(json["tamanho"], id.tamanho_dados);
}

#[test]
fn identificar_ps2() {
    let t = tempfile::tempdir().unwrap();
    let iso = comum::gravar(t.path(), "j.iso", &comum::iso_sintetica(&[("SYSTEM.CNF", b"BOOT2 = cdrom0:\\SCUS_972.65;1\r\n"), ("SCUS_972.65", b"ELF")]));
    let (codigo, json, _) = cli(&["identificar", s(&iso), "--plataforma", "ps2"]);
    assert_eq!(codigo, 0, "{json}");
    assert_eq!(json["serial"], "SCUS-97265");
    assert_eq!(json["tamanho"], fs::metadata(&iso).unwrap().len());
}

#[test]
fn identificar_arquivo_errado() {
    let t = tempfile::tempdir().unwrap();
    let nada = comum::gravar(t.path(), "nada.cue", b"lixo");
    let (codigo, json, _) = cli(&["identificar", s(&nada), "--plataforma", "ps1"]);
    assert_eq!(codigo, 1);
    assert_eq!(json["ok"], false);
}

#[test]
fn instalar_ps1() {
    let c = ps1();
    let x = xdelta3();
    let (codigo, json, erro) = cli(&["instalar", s(&c.pacote), "--original", s(&c.cue), "--xdelta3", s(&x)]);
    assert_eq!(codigo, 0, "{json} {erro}");
    assert_eq!(json["compativel_ra"], true);
    let saida = c.jogo.join("Jogo de PS1 (USA) PT-BR v1.0.cue");
    assert_eq!(json["saida"], s(&saida));
    assert_eq!(fs::read(c.jogo.join("Jogo de PS1 (USA) PT-BR v1.0.bin")).unwrap(), c.destino);
    assert!(erro.contains("aplicando"), "o progresso vai para a saída de erro: {erro}");
    // De novo, sem --sobrescrever: recusa.
    let (codigo, json, _) = cli(&["instalar", s(&c.pacote), "--original", s(&c.cue), "--xdelta3", s(&x)]);
    assert_eq!(codigo, 1);
    assert!(json["erro"].as_str().unwrap().contains("Já existe"), "{json}");
    let (codigo, _, _) = cli(&["instalar", s(&c.pacote), "--original", s(&c.cue), "--xdelta3", s(&x), "--sobrescrever"]);
    assert_eq!(codigo, 0);
}

#[test]
fn instalar_e_remover_pc() {
    let t = tempfile::tempdir().unwrap();
    let (jogo, pacote) = pc(t.path());
    let (codigo, json, _) = cli(&["instalar", s(&pacote), "--original", s(&jogo), "--xdelta3", "nao-usado"]);
    assert_eq!(codigo, 0, "{json}");
    assert_eq!(json["saida"], s(&jogo));
    assert_eq!(fs::read(jogo.join("dados").join("menu.txt")).unwrap(), b"MENU PT");
    assert_eq!(fs::read(jogo.join("mods").join("carregador.dll")).unwrap(), b"DLL");
    let (codigo, json, _) = cli(&["remover", s(&pacote), "--pasta", s(&jogo)]);
    assert_eq!(codigo, 0, "{json}");
    assert_eq!(json["removidos"], 1);
    assert_eq!(json["restaurados"], 1);
    assert_eq!(fs::read(jogo.join("dados").join("menu.txt")).unwrap(), b"MENU EN");
    assert!(!jogo.join("mods").exists());
}

#[test]
fn previa_gera_lista() {
    let c = ps1();
    let (_, pc_pacote) = pc(c.pacote.parent().unwrap().parent().unwrap());
    assert_eq!(pc_pacote.parent(), c.pacote.parent());
    let saida = c.jogo.parent().unwrap().join("previa");
    let (codigo, json, _) = cli(&["previa", s(c.pacote.parent().unwrap()), "--saida", s(&saida)]);
    assert_eq!(codigo, 0, "{json}");
    assert_eq!(json, serde_json::json!({ "ok": true, "jogos": 2 }));
    let lista: Value = serde_json::from_str(&fs::read_to_string(saida.join("lista.json")).unwrap()).unwrap();
    assert_eq!(lista["jogos"].as_array().unwrap().len(), 2);
    assert_eq!(lista["mostrar_trailers"], true);
    assert_eq!(lista["atualizacoes"], serde_json::json!([]));
    assert_eq!(lista["catalogo"], serde_json::json!({ "offline": false, "atualizado_em": null, "aviso": null }));
    let traducoes: Vec<&Value> = lista["jogos"].as_array().unwrap().iter().map(|j| &j["plataformas"][0]["traducoes"][0]).collect();
    assert!(traducoes.iter().any(|t| t["ra"] == "compativel"));
    assert!(traducoes.iter().any(|t| t["contem_programa"] == true));
}

#[test]
fn previa_com_pacote_ruim_recusa() {
    let c = ps1();
    fs::write(c.pacote.join("ficha.json"), "{").unwrap();
    let saida = c.jogo.parent().unwrap().join("previa");
    let (codigo, json, _) = cli(&["previa", s(c.pacote.parent().unwrap()), "--saida", s(&saida)]);
    assert_eq!(codigo, 1);
    assert_eq!(json["ok"], false);
}
