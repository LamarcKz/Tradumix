//! Instalar e remover tradução de PC na pasta do jogo.
mod comum;

use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use nucleo::{Erro, Etapa, Pacote, Pedido, aplicar, aplicar_com_rede, carregar_pacotes, conferir, instalado_pc, remover_pc};

const EXE: &[u8] = b"EXECUTAVEL DO JOGO v1.0.3";
const ID: &str = "jogo-pc-ptbr";

fn zip_de(entradas: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opcoes = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (nome, dados) in entradas {
        zip.start_file(*nome, opcoes).unwrap();
        zip.write_all(dados).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

struct Cena {
    _t: tempfile::TempDir,
    raiz: PathBuf,
    jogo: PathBuf,
}

impl Cena {
    /// Pasta do jogo com `jogo.exe` e os arquivos dados.
    fn nova(existentes: &[(&str, &[u8])]) -> Cena {
        let t = tempfile::tempdir().unwrap();
        let raiz = t.path().to_path_buf();
        let jogo = raiz.join("Steam").join("Jogo de PC");
        fs::create_dir_all(&jogo).unwrap();
        fs::write(jogo.join("jogo.exe"), EXE).unwrap();
        for (caminho, dados) in existentes {
            escrever(&jogo, caminho, dados);
        }
        Cena { _t: t, raiz, jogo }
    }

    /// Pacote de uma versão da tradução: a ficha lista `arquivos` e o `.zip` tem `no_zip`
    /// (por padrão, os mesmos arquivos).
    fn pacote(&self, versao: &str, arquivos: &[(&str, &[u8])], no_zip: Option<&[(&str, &[u8])]>) -> Pacote {
        let zip = zip_de(no_zip.unwrap_or(arquivos));
        let lista: Vec<String> = arquivos
            .iter()
            .map(|(c, d)| format!(r#"{{"caminho":"{c}","sha256":"{}","tamanho":{}}}"#, comum::sha256_hex(d), d.len()))
            .collect();
        let refs = format!(r#"[{{"caminho":"jogo.exe","sha256":"{}"}}]"#, comum::sha256_hex(EXE));
        let ficha = comum::ficha_pc(ID, versao, &comum::sha256_hex(&zip), zip.len() as u64, &format!("[{}]", lista.join(",")), &refs);
        let pasta = self.raiz.join(format!("pacotes-{versao}")).join(ID);
        fs::create_dir_all(&pasta).unwrap();
        fs::write(pasta.join("ficha.json"), ficha).unwrap();
        fs::write(pasta.join("pacote.zip"), &zip).unwrap();
        let carga = carregar_pacotes(pasta.parent().unwrap());
        assert!(carga.invalidos.is_empty(), "{:?}", carga.invalidos);
        carga.pacotes.into_iter().next().unwrap()
    }

    fn pedido(&self) -> Pedido {
        Pedido { iso_original: self.jogo.clone(), saida: self.jogo.clone(), sobrescrever: false, xdelta3: PathBuf::from("nao-usado") }
    }

    fn instalar(&self, pacote: &Pacote) -> Result<nucleo::Resultado, Erro> {
        aplicar(pacote, &self.pedido(), &mut |_, _| {}, &AtomicBool::new(false))
    }

    fn ler(&self, caminho: &str) -> Option<Vec<u8>> {
        fs::read(self.jogo.join(caminho)).ok()
    }

    /// Todos os arquivos da pasta do jogo (fora o registro `.tradumix`) com o conteúdo.
    fn foto(&self) -> BTreeMap<String, Vec<u8>> {
        let mut m = BTreeMap::new();
        foto_em(&self.jogo, &self.jogo, &mut m);
        m
    }
}

fn foto_em(base: &Path, pasta: &Path, m: &mut BTreeMap<String, Vec<u8>>) {
    for e in fs::read_dir(pasta).unwrap() {
        let p = e.unwrap().path();
        let rel = p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
        if rel == ".tradumix" {
            continue;
        }
        if p.is_dir() {
            m.insert(format!("{rel}/"), Vec::new());
            foto_em(base, &p, m);
        } else {
            m.insert(rel, fs::read(&p).unwrap());
        }
    }
}

fn escrever(pasta: &Path, caminho: &str, dados: &[u8]) {
    let p = pasta.join(caminho);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, dados).unwrap();
}

fn registro(c: &Cena) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(c.jogo.join(".tradumix").join(ID).join("instalado.json")).unwrap()).unwrap()
}

#[test]
fn instala_arquivos_novos() {
    let c = Cena::nova(&[]);
    let p = c.pacote("1.0", &[("mods/textos.sub", b"TEXTOS PT")], None);
    let mut etapas: Vec<Etapa> = Vec::new();
    let r = aplicar(&p, &c.pedido(), &mut |e, _| if etapas.last() != Some(&e) { etapas.push(e) }, &AtomicBool::new(false)).unwrap();
    assert_eq!(etapas, [Etapa::Conferindo, Etapa::Aplicando, Etapa::Verificando]);
    assert_eq!(r.saida, c.jogo);
    assert_eq!(r.hash_ra, None);
    assert_eq!(r.compativel_ra, None);
    assert_eq!(c.ler("mods/textos.sub").unwrap(), b"TEXTOS PT");
    let reg = registro(&c);
    assert_eq!(reg["id"], ID);
    assert_eq!(reg["versao"], "1.0");
    assert_eq!(reg["arquivos"][0]["caminho"], "mods/textos.sub");
    assert_eq!(reg["arquivos"][0]["original"], false);
    assert_eq!(instalado_pc(&p, &c.jogo).as_deref(), Some("1.0"));
    // Nada sobra da extração.
    assert!(!c.jogo.join(".tradumix").join(ID).join("novo").exists());
}

#[test]
fn guarda_original_ao_substituir() {
    let c = Cena::nova(&[("dados/menu.txt", b"MENU EN")]);
    let p = c.pacote("1.0", &[("dados/menu.txt", b"MENU PT")], None);
    c.instalar(&p).unwrap();
    assert_eq!(c.ler("dados/menu.txt").unwrap(), b"MENU PT");
    let guardado = c.jogo.join(".tradumix").join(ID).join("originais").join("dados").join("menu.txt");
    assert_eq!(fs::read(guardado).unwrap(), b"MENU EN");
    assert_eq!(registro(&c)["arquivos"][0]["original"], true);
}

#[test]
fn recusa_substituir_exe_ou_dll_do_jogo() {
    for alvo in ["jogo.exe", "bin/motor.dll"] {
        let c = Cena::nova(&[("bin/motor.dll", b"DLL DO JOGO"), ("dados/menu.txt", b"MENU EN")]);
        let antes = c.foto();
        let p = c.pacote("1.0", &[("dados/menu.txt", b"MENU PT"), (alvo, b"TROCADO")], None);
        let r = c.instalar(&p);
        assert!(matches!(&r, Err(Erro::ArquivoDoJogoProtegido(x)) if x == alvo), "{r:?}");
        assert!(r.unwrap_err().mensagem().contains(alvo));
        assert_eq!(c.foto(), antes, "{alvo}");
    }
}

#[test]
fn aceita_dll_nova() {
    let c = Cena::nova(&[]);
    let p = c.pacote("1.0", &[("mods/carregador.dll", b"DLL NOVA")], None);
    assert!(p.ficha.contem_programa());
    c.instalar(&p).unwrap();
    assert_eq!(c.ler("mods/carregador.dll").unwrap(), b"DLL NOVA");
}

#[test]
fn zip_diferente_da_lista() {
    let c = Cena::nova(&[("dados/menu.txt", b"MENU EN")]);
    let antes = c.foto();
    // Um arquivo a mais no .zip.
    let p = c.pacote("1.0", &[("dados/menu.txt", b"MENU PT")], Some(&[("dados/menu.txt", b"MENU PT"), ("extra.txt", b"x")]));
    assert!(c.instalar(&p).is_err());
    assert_eq!(c.foto(), antes);
    // Um arquivo faltando no .zip.
    let p = c.pacote("1.1", &[("dados/menu.txt", b"MENU PT"), ("b.txt", b"b")], Some(&[("dados/menu.txt", b"MENU PT")]));
    assert!(c.instalar(&p).is_err());
    assert_eq!(c.foto(), antes);
    // Conteúdo diferente do sha256 da ficha.
    let p = c.pacote("1.2", &[("dados/menu.txt", b"MENU PT")], Some(&[("dados/menu.txt", b"OUTRO")]));
    assert!(c.instalar(&p).is_err());
    assert_eq!(c.foto(), antes);
}

#[test]
fn zip_com_caminho_malicioso() {
    for ruim in ["../../fora.txt", "../fora.txt", "C:/fora.txt", "/fora.txt", "a/../../fora.txt"] {
        let c = Cena::nova(&[]);
        let antes = c.foto();
        let p = c.pacote("1.0", &[("a.txt", b"a")], Some(&[("a.txt", b"a"), (ruim, b"MAL")]));
        assert!(c.instalar(&p).is_err(), "{ruim}");
        assert_eq!(c.foto(), antes, "{ruim}");
        assert!(!c.raiz.join("fora.txt").exists(), "{ruim}");
        assert!(!c.jogo.parent().unwrap().join("fora.txt").exists(), "{ruim}");
    }
}

#[test]
fn versao_diferente_avisa() {
    let c = Cena::nova(&[]);
    let p = c.pacote("1.0", &[("a.txt", b"a")], None);
    fs::write(c.jogo.join("jogo.exe"), b"EXECUTAVEL v2").unwrap();
    let aviso = conferir(&p, &c.jogo).unwrap().expect("devia avisar");
    assert_eq!(aviso, "Esta pasta tem uma versão do jogo diferente da testada (1.0.3). A tradução pode não funcionar direito.");
    c.instalar(&p).unwrap();
    assert_eq!(c.ler("a.txt").unwrap(), b"a");
}

#[test]
fn versao_certa_nao_avisa() {
    let c = Cena::nova(&[]);
    let p = c.pacote("1.0", &[("a.txt", b"a")], None);
    assert_eq!(conferir(&p, &c.jogo).unwrap(), None);
}

#[test]
fn pasta_errada() {
    let c = Cena::nova(&[]);
    let p = c.pacote("1.0", &[("a.txt", b"a")], None);
    let outra = c.raiz.join("outra pasta");
    fs::create_dir_all(&outra).unwrap();
    let erro = conferir(&p, &outra).unwrap_err();
    assert!(matches!(erro, Erro::PastaDoJogoErrada { .. }));
    assert_eq!(erro.mensagem(), "Esta pasta não parece ser do Jogo de PC.");
    let mut pedido = c.pedido();
    pedido.iso_original = outra.clone();
    pedido.saida = outra.clone();
    assert!(aplicar(&p, &pedido, &mut |_, _| {}, &AtomicBool::new(false)).is_err());
    assert_eq!(fs::read_dir(&outra).unwrap().count(), 0);
}

#[test]
fn remover_restaura_original() {
    let c = Cena::nova(&[("dados/menu.txt", b"MENU EN")]);
    let antes = c.foto();
    let p = c.pacote("1.0", &[("dados/menu.txt", b"MENU PT"), ("mods/textos.sub", b"TEXTOS")], None);
    c.instalar(&p).unwrap();
    let r = remover_pc(&p, &c.jogo).unwrap();
    assert_eq!(r.removidos, 1);
    assert_eq!(r.restaurados, 1);
    assert!(r.mantidos.is_empty());
    assert_eq!(c.foto(), antes);
    assert!(!c.jogo.join(".tradumix").exists());
    assert_eq!(instalado_pc(&p, &c.jogo), None);
}

#[test]
fn remover_mantem_arquivo_trocado_pela_loja() {
    let c = Cena::nova(&[("dados/menu.txt", b"MENU EN")]);
    let p = c.pacote("1.0", &[("dados/menu.txt", b"MENU PT")], None);
    c.instalar(&p).unwrap();
    fs::write(c.jogo.join("dados").join("menu.txt"), b"MENU EN v2 DA LOJA").unwrap();
    let r = remover_pc(&p, &c.jogo).unwrap();
    assert_eq!(r.mantidos, ["dados/menu.txt"]);
    assert_eq!(r.restaurados, 0);
    assert_eq!(c.ler("dados/menu.txt").unwrap(), b"MENU EN v2 DA LOJA");
}

#[test]
fn remover_sem_instalacao() {
    let c = Cena::nova(&[]);
    let p = c.pacote("1.0", &[("a.txt", b"a")], None);
    let erro = remover_pc(&p, &c.jogo).unwrap_err();
    assert_eq!(erro.mensagem(), "Esta tradução não está instalada nesta pasta.");
}

#[test]
fn reinstalar_nao_perde_original() {
    let c = Cena::nova(&[("dados/menu.txt", b"MENU EN")]);
    let antes = c.foto();
    let v1 = c.pacote("1.0", &[("dados/menu.txt", b"MENU PT 1.0")], None);
    c.instalar(&v1).unwrap();
    let v11 = c.pacote("1.1", &[("dados/menu.txt", b"MENU PT 1.1"), ("novo.txt", b"n")], None);
    c.instalar(&v11).unwrap();
    assert_eq!(c.ler("dados/menu.txt").unwrap(), b"MENU PT 1.1");
    assert_eq!(instalado_pc(&v11, &c.jogo).as_deref(), Some("1.1"));
    let guardado = c.jogo.join(".tradumix").join(ID).join("originais").join("dados").join("menu.txt");
    assert_eq!(fs::read(guardado).unwrap(), b"MENU EN");
    remover_pc(&v11, &c.jogo).unwrap();
    assert_eq!(c.foto(), antes);
}

#[test]
fn reinstalar_tira_arquivo_que_saiu_da_versao_nova() {
    let c = Cena::nova(&[("dados/menu.txt", b"MENU EN")]);
    let antes = c.foto();
    let v1 = c.pacote("1.0", &[("dados/menu.txt", b"MENU PT"), ("velho.txt", b"v")], None);
    c.instalar(&v1).unwrap();
    let v11 = c.pacote("1.1", &[("novo.txt", b"n")], None);
    c.instalar(&v11).unwrap();
    assert_eq!(c.ler("velho.txt"), None);
    assert_eq!(c.ler("dados/menu.txt").unwrap(), b"MENU EN");
    remover_pc(&v11, &c.jogo).unwrap();
    assert_eq!(c.foto(), antes);
}

#[test]
fn cancelar_no_meio() {
    let c = Cena::nova(&[("a.txt", b"A EN"), ("b.txt", b"B EN"), ("c.txt", b"C EN")]);
    let antes = c.foto();
    let p = c.pacote("1.0", &[("a.txt", b"A PT"), ("b.txt", b"B PT"), ("c.txt", b"C PT"), ("d.txt", b"D")], None);
    let cancelar = AtomicBool::new(false);
    let r = aplicar(&p, &c.pedido(), &mut |e, f| {
        // Cancela depois da cópia do primeiro arquivo.
        if e == Etapa::Aplicando && f > 0.0 {
            cancelar.store(true, Ordering::Relaxed);
        }
    }, &cancelar);
    assert!(matches!(r, Err(Erro::Cancelado)), "{r:?}");
    assert_eq!(c.foto(), antes);
    assert!(!c.jogo.join(".tradumix").exists());
}

#[test]
fn baixa_o_pacote_antes() {
    let c = Cena::nova(&[]);
    let p = c.pacote("1.0", &[("a.txt", b"a")], None);
    let servidor = comum::servidor::Servidor::novo();
    servidor.definir("/pacote.zip", comum::servidor::Resposta::Ok(fs::read(p.patch()).unwrap()));
    let ficha_path = p.pasta.join("ficha.json");
    let ficha = fs::read_to_string(&ficha_path).unwrap().replace("https://x/pacote.zip", &servidor.url("/pacote.zip"));
    fs::write(&ficha_path, ficha).unwrap();
    fs::remove_file(p.patch()).unwrap();
    let p = carregar_pacotes(p.pasta.parent().unwrap()).pacotes.remove(0);
    let rede = nucleo::RedeUreq::novo();
    let mut etapas: Vec<Etapa> = Vec::new();
    aplicar_com_rede(&p, &c.pedido(), Some(&rede), &mut |e, _| if etapas.last() != Some(&e) { etapas.push(e) }, &AtomicBool::new(false)).unwrap();
    assert_eq!(etapas, [Etapa::Conferindo, Etapa::Baixando, Etapa::Aplicando, Etapa::Verificando]);
    assert_eq!(c.ler("a.txt").unwrap(), b"a");
}
