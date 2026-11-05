//! Comandos que a interface chama. Cada um é uma função interna testável mais um envoltório Tauri.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use base64::Engine;
use nucleo::{Atualizacao, Erro, Estado as EstadoTraducao, Historico, Imagem, Pacote, Pedido, Resultado, Tradutor};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::config::Config;

/// Estado compartilhado do app.
pub struct Estado {
    dados: PathBuf,
    config: Mutex<Config>,
    cancelar: Arc<AtomicBool>,
    traduzindo: Arc<AtomicBool>,
    fechar_ao_terminar: Arc<AtomicBool>,
}

impl Estado {
    pub fn novo(dados: PathBuf) -> Estado {
        let config = Config::carregar(&dados.join("config.json"));
        Estado {
            dados,
            config: Mutex::new(config),
            cancelar: Arc::new(AtomicBool::new(false)),
            traduzindo: Arc::new(AtomicBool::new(false)),
            fechar_ao_terminar: Arc::new(AtomicBool::new(false)),
        }
    }

    /// A janela pediu para fechar. Com uma tradução em andamento, cancela e devolve `true`:
    /// o fechamento espera o núcleo apagar a ISO parcial e acontece no fim de `traduzir`.
    pub fn pedir_fechamento(&self) -> bool {
        if !self.traduzindo.load(Ordering::SeqCst) {
            return false;
        }
        self.fechar_ao_terminar.store(true, Ordering::SeqCst);
        self.cancelar.store(true, Ordering::SeqCst);
        true
    }

    fn pasta_rascunhos(&self) -> Option<PathBuf> {
        self.config.lock().unwrap().pasta_rascunhos.clone()
    }

    fn arquivo_historico(&self) -> PathBuf {
        self.dados.join("historico.json")
    }
}

#[derive(Debug, Serialize)]
pub struct TraducaoDto {
    pub id: String,
    pub versao: String,
    pub estado: EstadoTraducao,
    pub idioma: String,
    pub regiao: String,
    pub serial: String,
    pub tradutores: Vec<Tradutor>,
    pub permissao: Option<String>,
    pub descricao: String,
    pub traduzido: Vec<String>,
    pub limitacoes: Vec<String>,
    pub data: String,
    pub fundo: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JogoDto {
    pub id: String,
    pub nome: String,
    pub console: String,
    pub capa: Option<String>,
    pub traducoes: Vec<TraducaoDto>,
}

#[derive(Debug, Serialize)]
pub struct InvalidoDto {
    pub pasta: String,
    pub motivo: String,
}

#[derive(Debug, Serialize)]
pub struct ListaJogos {
    pub jogos: Vec<JogoDto>,
    pub invalidos: Vec<InvalidoDto>,
    pub atualizacoes: Vec<Atualizacao>,
    pub pasta_rascunhos: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConferenciaDto {
    pub serve: bool,
    pub mensagem: Option<String>,
    pub saida_sugerida: String,
    pub ja_existe: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Progresso {
    etapa: nucleo::Etapa,
    fracao: f32,
}

fn data_url(caminho: &Path) -> Option<String> {
    let bytes = std::fs::read(caminho).ok()?;
    let ext = caminho.extension()?.to_string_lossy().to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => return None,
    };
    Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

fn carregar(pasta: Option<&Path>) -> nucleo::CargaPacotes {
    pasta.map(nucleo::carregar_pacotes).unwrap_or_default()
}

pub fn montar_lista(pasta: Option<&Path>, historico: &Historico) -> ListaJogos {
    let carga = carregar(pasta);
    let jogos = nucleo::agrupar_por_jogo(&carga.pacotes)
        .into_iter()
        .map(|j| {
            let pacotes: Vec<&Pacote> = j.pacotes.iter().filter_map(|id| carga.pacotes.iter().find(|p| &p.ficha.id == id)).collect();
            let capa = pacotes.iter().find_map(|p| p.imagem(Imagem::Capa).and_then(|c| data_url(&c)));
            let traducoes = pacotes
                .iter()
                .map(|p| {
                    let f = &p.ficha;
                    TraducaoDto {
                        id: f.id.clone(),
                        versao: f.versao.clone(),
                        estado: f.estado,
                        idioma: f.idioma.clone(),
                        regiao: f.origem.regiao.nome().to_string(),
                        serial: f.origem.serial.clone(),
                        tradutores: f.tradutores.clone(),
                        permissao: f.permissao.clone(),
                        descricao: f.descricao.clone(),
                        traduzido: f.traduzido.clone(),
                        limitacoes: f.limitacoes.clone(),
                        data: f.data.clone(),
                        fundo: p.imagem(Imagem::Fundo).and_then(|c| data_url(&c)),
                    }
                })
                .collect();
            JogoDto { id: j.id, nome: j.nome, console: j.console, capa, traducoes }
        })
        .collect();
    ListaJogos {
        jogos,
        invalidos: carga
            .invalidos
            .into_iter()
            .map(|i| InvalidoDto { pasta: i.pasta.to_string_lossy().into_owned(), motivo: i.motivo })
            .collect(),
        atualizacoes: historico.atualizacoes(&carga.pacotes),
        pasta_rascunhos: pasta.map(|p| p.to_string_lossy().into_owned()),
    }
}

pub fn conferir_interno(pacotes: &[Pacote], pacote_id: &str, iso: &Path) -> Result<ConferenciaDto, String> {
    let pacote = pacotes.iter().find(|p| p.ficha.id == pacote_id).ok_or("Tradução não encontrada.")?;
    let saida = nucleo::nome_saida(&pacote.ficha, iso);
    let (serve, mensagem) = match nucleo::conferir(pacote, iso) {
        Ok(_) => (true, None),
        Err(e) => (false, Some(e.mensagem())),
    };
    Ok(ConferenciaDto {
        serve,
        mensagem,
        ja_existe: saida.exists(),
        saida_sugerida: saida.to_string_lossy().into_owned(),
    })
}

/// O xdelta3 vai ao lado do executável do app (sidecar do Tauri).
fn caminho_xdelta3() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let caminho = exe.parent().ok_or("pasta do app desconhecida")?.join("xdelta3.exe");
    if caminho.is_file() { Ok(caminho) } else { Err("O xdelta3 não foi encontrado junto do app.".into()) }
}

#[tauri::command]
pub fn listar_jogos(estado: State<'_, Estado>) -> ListaJogos {
    let historico = Historico::carregar(&estado.arquivo_historico());
    montar_lista(estado.pasta_rascunhos().as_deref(), &historico)
}

#[tauri::command]
pub fn escolher_pasta_rascunhos(estado: State<'_, Estado>, caminho: String) -> Result<ListaJogos, String> {
    {
        let mut config = estado.config.lock().unwrap();
        config.pasta_rascunhos = Some(PathBuf::from(&caminho));
        config.salvar(&estado.dados.join("config.json"))?;
    }
    Ok(listar_jogos(estado))
}

#[tauri::command]
pub fn conferir_iso(estado: State<'_, Estado>, pacote_id: String, iso: String) -> Result<ConferenciaDto, String> {
    let carga = carregar(estado.pasta_rascunhos().as_deref());
    conferir_interno(&carga.pacotes, &pacote_id, Path::new(&iso))
}

#[tauri::command]
pub async fn traduzir(
    app: AppHandle,
    estado: State<'_, Estado>,
    pacote_id: String,
    iso: String,
    saida: String,
    sobrescrever: bool,
) -> Result<Resultado, String> {
    if estado.traduzindo.swap(true, Ordering::SeqCst) {
        return Err("Já existe uma tradução em andamento.".into());
    }
    let traduzindo = estado.traduzindo.clone();
    let fechar_ao_terminar = estado.fechar_ao_terminar.clone();
    let app_fim = app.clone();
    let cancelar = estado.cancelar.clone();
    cancelar.store(false, Ordering::SeqCst);
    let pasta = estado.pasta_rascunhos();
    let arquivo_historico = estado.arquivo_historico();
    let tarefa = tauri::async_runtime::spawn_blocking(move || -> Result<Resultado, String> {
        let carga = carregar(pasta.as_deref());
        let pacote = carga.pacotes.into_iter().find(|p| p.ficha.id == pacote_id).ok_or("Tradução não encontrada.")?;
        let pedido = Pedido { iso_original: PathBuf::from(iso), saida: PathBuf::from(saida), sobrescrever, xdelta3: caminho_xdelta3()? };
        let resultado = nucleo::aplicar(
            &pacote,
            &pedido,
            &mut |etapa, fracao| {
                let _ = app.emit("progresso", Progresso { etapa, fracao });
            },
            &cancelar,
        )
        .map_err(|e: Erro| e.mensagem())?;
        let mut historico = Historico::carregar(&arquivo_historico);
        historico.registrar(&pacote, &resultado, &chrono::Local::now().format("%Y-%m-%d").to_string());
        let _ = historico.salvar(&arquivo_historico);
        Ok(resultado)
    });
    let resultado = tarefa.await.map_err(|e| e.to_string()).and_then(|r| r);
    traduzindo.store(false, Ordering::SeqCst);
    if fechar_ao_terminar.load(Ordering::SeqCst) {
        app_fim.exit(0);
    }
    resultado
}

#[tauri::command]
pub fn cancelar_traducao(estado: State<'_, Estado>) {
    estado.cancelar.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub fn arquivo_existe(caminho: String) -> bool {
    Path::new(&caminho).exists()
}

#[tauri::command]
pub fn abrir_pasta(caminho: String) -> Result<(), String> {
    tauri_plugin_opener::reveal_item_in_dir(PathBuf::from(caminho)).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "../../crates/nucleo/tests/comum/mod.rs"]
mod comum;

#[cfg(test)]
mod testes {
    use super::*;
    use std::fs;

    const CNF: &[u8] = b"BOOT2 = cdrom0:\\SCUS_972.65;1\r\n";

    fn pasta_com_pacote(raiz: &Path, hash: &str) {
        pasta_com_pacote_de_tamanho(raiz, hash, 40960)
    }

    fn pasta_com_pacote_de_tamanho(raiz: &Path, hash: &str, tamanho: u64) {
        let p = raiz.join("jak2");
        fs::create_dir_all(&p).unwrap();
        fs::write(p.join("patch.xdelta"), b"VCD").unwrap();
        fs::write(p.join("capa.png"), [0x89, b'P', b'N', b'G']).unwrap();
        fs::write(
            p.join("ficha.json"),
            format!(
                r#"{{"formato":1,"id":"jak2-usa-ptbr","jogo":{{"id":"jak2","nome":"Jak II","console":"ps2"}},
                "versao":"1.0","estado":"rascunho","idioma":"pt-BR","data":"2026-11-05",
                "origem":{{"serial":"SCUS-97265","regiao":"usa","hash_ra":"{hash}","tamanho":{tamanho}}},
                "resultado":{{"hash_ra":"{hash}","tamanho":{tamanho}}},
                "tradutores":[{{"nome":"Fulano"}}],"arquivos":{{"patch":"patch.xdelta","capa":"capa.png"}}}}"#
            ),
        )
        .unwrap();
    }

    #[test]
    fn lista_jogos_de_pasta_de_rascunhos() {
        let raiz = tempfile::tempdir().unwrap();
        pasta_com_pacote(raiz.path(), "f51c94dd47cc2eb385d5733389428121");
        let lista = montar_lista(Some(raiz.path()), &Historico::default());
        assert_eq!(lista.jogos.len(), 1);
        assert_eq!(lista.jogos[0].nome, "Jak II");
        assert_eq!(lista.jogos[0].traducoes.len(), 1);
        assert_eq!(lista.jogos[0].traducoes[0].regiao, "USA");
        assert!(lista.jogos[0].capa.as_deref().unwrap().starts_with("data:image/png;base64,"));
        assert_eq!(lista.pasta_rascunhos.as_deref(), Some(raiz.path().to_string_lossy().as_ref()));
    }

    /// Os links dos créditos abrem no navegador pelo plugin opener; sem esta permissão o clique não faz nada.
    #[test]
    fn janela_pode_abrir_links_dos_creditos() {
        let texto = include_str!("../capabilities/default.json");
        let json: serde_json::Value = serde_json::from_str(texto).unwrap();
        let permissoes: Vec<&str> = json["permissions"].as_array().unwrap().iter().filter_map(|p| p.as_str()).collect();
        assert!(permissoes.contains(&"opener:allow-open-url"), "{permissoes:?}");
    }

    #[test]
    fn fechar_durante_traducao_cancela_e_adia() {
        let dados = tempfile::tempdir().unwrap();
        let estado = Estado::novo(dados.path().to_path_buf());
        assert!(!estado.pedir_fechamento(), "sem tradução, fecha na hora");
        estado.traduzindo.store(true, Ordering::SeqCst);
        assert!(estado.pedir_fechamento(), "com tradução, adia o fechamento");
        assert!(estado.cancelar.load(Ordering::SeqCst));
        assert!(estado.fechar_ao_terminar.load(Ordering::SeqCst));
    }

    #[test]
    fn sem_pasta_a_lista_fica_vazia() {
        let lista = montar_lista(None, &Historico::default());
        assert!(lista.jogos.is_empty());
        assert!(lista.pasta_rascunhos.is_none());
    }

    #[test]
    fn conferir_iso_de_outro_jogo_devolve_mensagem() {
        let raiz = tempfile::tempdir().unwrap();
        pasta_com_pacote(raiz.path(), "f51c94dd47cc2eb385d5733389428121");
        let outra = comum::gravar(
            raiz.path(),
            "outra.iso",
            &comum::iso_sintetica(&[("SYSTEM.CNF", b"BOOT2 = cdrom0:\\SLES_123.45;1\r\n"), ("SLES_123.45", b"ELF")]),
        );
        let pacotes = nucleo::carregar_pacotes(raiz.path()).pacotes;
        let c = conferir_interno(&pacotes, "jak2-usa-ptbr", &outra).unwrap();
        assert!(!c.serve);
        assert!(c.mensagem.unwrap().contains("SCUS-97265"));
    }

    #[test]
    fn conferir_iso_certa_sugere_saida() {
        let raiz = tempfile::tempdir().unwrap();
        let iso = comum::iso_sintetica(&[("SYSTEM.CNF", CNF), ("SCUS_972.65", b"ELF")]);
        let caminho = comum::gravar(raiz.path(), "original.iso", &iso);
        let hash = nucleo::identificar(&caminho).unwrap().hash_ra;
        pasta_com_pacote_de_tamanho(&raiz.path().join("pacotes"), &hash, iso.len() as u64);
        let pacotes = nucleo::carregar_pacotes(&raiz.path().join("pacotes")).pacotes;
        let c = conferir_interno(&pacotes, "jak2-usa-ptbr", &caminho).unwrap();
        assert!(c.serve, "{:?}", c.mensagem);
        assert_eq!(c.saida_sugerida, raiz.path().join("Jak II (USA) PT-BR v1.0.iso").to_string_lossy());
        assert!(!c.ja_existe);
    }
}
