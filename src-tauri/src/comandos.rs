//! Comandos que a interface chama. Cada um é uma função interna testável mais um envoltório Tauri.
//!
//! O app mostra só as traduções do catálogo (sincronizado para `<dados>/catalogo`). As funções de
//! lista ainda aceitam uma pasta de rascunhos (`rascunho:<id>`), usada nos testes e guardada para a
//! futura ferramenta de publicação; o app sempre passa `None`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use base64::Engine;
use nucleo::{Atualizacao, Entrada, Erro, Estado as EstadoTraducao, Historico, Imagem, Pacote, Pedido, RedeUreq, Resultado, Sincronizacao, Tradutor};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::banco::Banco;

const URL_CATALOGO: &str = "https://raw.githubusercontent.com/LamarcKz/Tradumix-catalogo/main/indice.json";
const PREFIXO_RASCUNHO: &str = "rascunho:";

/// Estado compartilhado do app.
pub struct Estado {
    dados: PathBuf,
    banco: Banco,
    sincronizacao: Mutex<Option<Sincronizacao>>,
    cancelar: Arc<AtomicBool>,
    traduzindo: Arc<AtomicBool>,
    sincronizando: AtomicBool,
    /// Protege a troca conjunta de `traduzindo` e `sincronizando`.
    trava: Mutex<()>,
    fechar_ao_terminar: Arc<AtomicBool>,
    vistos: Vistos,
}

/// O selo "Novo" só aparece na primeira abertura do app em que a tradução aparece (e só dentro dos 14
/// dias da publicação). As já vistas ficam na tabela `vistos` do banco; o que já tinha sido visto
/// quando o app abriu perde o selo, e o que aparece pela primeira vez fica com ele até fechar.
pub struct Vistos {
    no_inicio: HashSet<String>,
}

impl Vistos {
    pub fn carregar(banco: &Banco) -> Vistos {
        Vistos { no_inicio: banco.vistos() }
    }

    /// Tira o "Novo" do que já tinha sido visto e grava como visto tudo o que está na lista.
    pub fn marcar(&self, lista: &mut ListaJogos, banco: &Banco) {
        let mut ids = Vec::new();
        for t in lista.jogos.iter_mut().flat_map(|j| j.traducoes.iter_mut()) {
            if self.no_inicio.contains(&t.id) {
                t.novo = false;
            }
            ids.push(t.id.clone());
        }
        // Sem conseguir gravar, o pior que acontece é o selo aparecer de novo na próxima abertura.
        let _ = banco.marcar_vistos(&ids);
    }
}

impl Estado {
    pub fn novo(dados: PathBuf) -> Estado {
        // Sem conseguir abrir o banco na pasta de dados, o app funciona sem guardar nada.
        let banco = Banco::abrir(&dados).unwrap_or_else(|_| Banco::na_memoria());
        let vistos = Vistos::carregar(&banco);
        Estado {
            dados,
            banco,
            sincronizacao: Mutex::new(None),
            cancelar: Arc::new(AtomicBool::new(false)),
            traduzindo: Arc::new(AtomicBool::new(false)),
            sincronizando: AtomicBool::new(false),
            trava: Mutex::new(()),
            fechar_ao_terminar: Arc::new(AtomicBool::new(false)),
            vistos,
        }
    }

    /// A sincronização apaga e reescreve pastas do cache: não roda em dobro nem durante uma tradução.
    pub fn comecar_sincronizacao(&self) -> bool {
        let _t = self.trava.lock().unwrap();
        if self.traduzindo.load(Ordering::SeqCst) || self.sincronizando.load(Ordering::SeqCst) {
            return false;
        }
        self.sincronizando.store(true, Ordering::SeqCst);
        true
    }

    pub fn terminar_sincronizacao(&self) {
        self.sincronizando.store(false, Ordering::SeqCst);
    }

    /// Uma tradução por vez, e só com o catálogo parado.
    pub fn comecar_traducao(&self) -> bool {
        let _t = self.trava.lock().unwrap();
        if self.traduzindo.load(Ordering::SeqCst) || self.sincronizando.load(Ordering::SeqCst) {
            return false;
        }
        self.traduzindo.store(true, Ordering::SeqCst);
        true
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

    fn url_catalogo(&self) -> String {
        self.banco.preferencia("url_catalogo").unwrap_or_else(|| URL_CATALOGO.to_string())
    }

    fn cache(&self) -> PathBuf {
        self.dados.join("catalogo")
    }

    fn sincronizacao(&self) -> Sincronizacao {
        self.sincronizacao.lock().unwrap().clone().unwrap_or_else(|| sincronizacao_inicial(&self.cache()))
    }

    fn lista(&self) -> ListaJogos {
        let historico = self.banco.historico();
        let mut lista = montar_lista(&self.cache(), None, &historico, &self.sincronizacao(), &hoje());
        lista.mostrar_trailers = self.banco.preferencia("mostrar_trailers").as_deref() != Some("nao");
        self.vistos.marcar(&mut lista, &self.banco);
        lista
    }

    pub fn definir_trailers(&self, mostrar: bool) -> Result<(), String> {
        self.banco.definir_preferencia("mostrar_trailers", if mostrar { "sim" } else { "nao" })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TraducaoDto {
    pub id: String,
    pub origem: &'static str,
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
    pub novo: bool,
    pub patch_disponivel: bool,
    /// Tamanho do patch a baixar, quando ele ainda não está no PC.
    pub tamanho_download: Option<u64>,
    /// Id do trailer no YouTube.
    pub trailer_youtube: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JogoDto {
    pub id: String,
    pub nome: String,
    pub console: String,
    pub capa: Option<String>,
    pub traducoes: Vec<TraducaoDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvalidoDto {
    pub pasta: String,
    pub motivo: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListaJogos {
    pub jogos: Vec<JogoDto>,
    pub invalidos: Vec<InvalidoDto>,
    pub atualizacoes: Vec<Atualizacao>,
    pub pasta_rascunhos: Option<String>,
    pub catalogo: Sincronizacao,
    pub mostrar_trailers: bool,
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

fn hoje() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
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

/// Antes da primeira sincronização da sessão: a data da última que deu certo, sem aviso.
pub fn sincronizacao_inicial(cache: &Path) -> Sincronizacao {
    let atualizado_em = std::fs::read_to_string(cache.join("sincronizado.txt")).ok().map(|t| t.trim().to_string());
    Sincronizacao { offline: false, atualizado_em, aviso: None }
}

/// Sincroniza o catálogo. Endereços `http://127.0.0.1` só valem quando configurados de propósito (testes).
pub fn sincronizar_interno(cache: &Path, url: &str) -> Sincronizacao {
    let local = url.starts_with("http://127.0.0.1");
    nucleo::sincronizar(&RedeUreq::novo(), url, cache, local, Some(nucleo::ACERVO_CAPAS_PS2))
}

/// Todas as traduções, cada uma com o id que a interface usa e de onde veio.
fn todas(cache: &Path, rascunhos: Option<&Path>) -> (Vec<(String, &'static str, Pacote)>, Vec<nucleo::PacoteInvalido>) {
    let catalogo = nucleo::carregar_pacotes(cache);
    let mut lista: Vec<(String, &'static str, Pacote)> =
        catalogo.pacotes.into_iter().map(|p| (p.ficha.id.clone(), "catalogo", p)).collect();
    let mut invalidos = Vec::new();
    if let Some(pasta) = rascunhos {
        let carga = nucleo::carregar_pacotes(pasta);
        lista.extend(carga.pacotes.into_iter().map(|p| (format!("{PREFIXO_RASCUNHO}{}", p.ficha.id), "rascunho", p)));
        invalidos = carga.invalidos;
    }
    (lista, invalidos)
}

pub fn achar_pacote(cache: &Path, rascunhos: Option<&Path>, id: &str) -> Option<Pacote> {
    todas(cache, rascunhos).0.into_iter().find(|(i, _, _)| i == id).map(|(_, _, p)| p)
}

pub fn montar_lista(cache: &Path, rascunhos: Option<&Path>, historico: &Historico, sinc: &Sincronizacao, hoje: &str) -> ListaJogos {
    let (todas, invalidos) = todas(cache, rascunhos);
    let mut jogos: Vec<JogoDto> = Vec::new();
    for (id, origem, p) in &todas {
        let f = &p.ficha;
        let disponivel = p.patch_disponivel();
        let traducao = TraducaoDto {
            id: id.clone(),
            origem,
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
            novo: *origem == "catalogo" && nucleo::e_novo(f, hoje),
            patch_disponivel: disponivel,
            tamanho_download: if disponivel { None } else { f.download.as_ref().map(|d| d.patch.tamanho) },
            trailer_youtube: f.midia.as_ref().and_then(|m| m.trailer_youtube.clone()),
        };
        match jogos.iter_mut().find(|j| j.id == f.jogo.id) {
            Some(j) => {
                if j.capa.is_none() {
                    j.capa = p.imagem(Imagem::Capa).and_then(|c| data_url(&c));
                }
                j.traducoes.push(traducao);
            }
            None => jogos.push(JogoDto {
                id: f.jogo.id.clone(),
                nome: f.jogo.nome.clone(),
                console: f.jogo.console.clone(),
                capa: p.imagem(Imagem::Capa).and_then(|c| data_url(&c)),
                traducoes: vec![traducao],
            }),
        }
    }
    jogos.sort_by(|a, b| a.nome.to_lowercase().cmp(&b.nome.to_lowercase()));
    let pacotes: Vec<Pacote> = todas.iter().filter(|(_, o, _)| *o == "catalogo").map(|(_, _, p)| p.clone()).collect();
    ListaJogos {
        jogos,
        invalidos: invalidos.into_iter().map(|i| InvalidoDto { pasta: i.pasta.to_string_lossy().into_owned(), motivo: i.motivo }).collect(),
        atualizacoes: historico.atualizacoes(&pacotes),
        pasta_rascunhos: rascunhos.map(|p| p.to_string_lossy().into_owned()),
        catalogo: sinc.clone(),
        mostrar_trailers: true,
    }
}

pub fn conferir_interno(pacote: &Pacote, iso: &Path) -> ConferenciaDto {
    let saida = nucleo::nome_saida(&pacote.ficha, iso);
    let (serve, mensagem) = match nucleo::conferir(pacote, iso) {
        Ok(_) => (true, None),
        Err(e) => (false, Some(e.mensagem())),
    };
    ConferenciaDto { serve, mensagem, ja_existe: saida.exists(), saida_sugerida: saida.to_string_lossy().into_owned() }
}

/// O xdelta3 vai ao lado do executável do app (sidecar do Tauri).
fn caminho_xdelta3() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    xdelta3_na_pasta(exe.parent().ok_or("pasta do app desconhecida")?)
}

fn xdelta3_na_pasta(pasta: &Path) -> Result<PathBuf, String> {
    let caminho = pasta.join("xdelta3.exe");
    if caminho.is_file() {
        Ok(caminho)
    } else {
        Err("O componente que aplica a tradução (xdelta3) não foi encontrado. Reinstale o Tradumix.".into())
    }
}

#[tauri::command]
pub fn listar_jogos(estado: State<'_, Estado>) -> ListaJogos {
    estado.lista()
}

#[tauri::command]
pub async fn sincronizar_catalogo(estado: State<'_, Estado>) -> Result<ListaJogos, String> {
    if !estado.comecar_sincronizacao() {
        return Ok(estado.lista());
    }
    let (cache, url) = (estado.cache(), estado.url_catalogo());
    let sinc = tauri::async_runtime::spawn_blocking(move || sincronizar_interno(&cache, &url)).await;
    estado.terminar_sincronizacao();
    *estado.sincronizacao.lock().unwrap() = Some(sinc.map_err(|e| e.to_string())?);
    Ok(estado.lista())
}

#[tauri::command]
pub fn conferir_iso(estado: State<'_, Estado>, pacote_id: String, iso: String) -> Result<ConferenciaDto, String> {
    let pacote = achar_pacote(&estado.cache(), None, &pacote_id).ok_or("Tradução não encontrada.")?;
    Ok(conferir_interno(&pacote, Path::new(&iso)))
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
    if !estado.comecar_traducao() {
        return Err("Espere um instante: já há uma tradução em andamento ou o catálogo está sendo atualizado.".into());
    }
    let traduzindo = estado.traduzindo.clone();
    let fechar_ao_terminar = estado.fechar_ao_terminar.clone();
    let app_fim = app.clone();
    let cancelar = estado.cancelar.clone();
    cancelar.store(false, Ordering::SeqCst);
    let cache = estado.cache();
    let tarefa = tauri::async_runtime::spawn_blocking(move || -> Result<(Resultado, Entrada), String> {
        let pacote = achar_pacote(&cache, None, &pacote_id).ok_or("Tradução não encontrada.")?;
        let pedido = Pedido { iso_original: PathBuf::from(iso), saida: PathBuf::from(saida), sobrescrever, xdelta3: caminho_xdelta3()? };
        let rede = RedeUreq::novo();
        let resultado = nucleo::aplicar_com_rede(
            &pacote,
            &pedido,
            Some(&rede),
            &mut |etapa, fracao| {
                let _ = app.emit("progresso", Progresso { etapa, fracao });
            },
            &cancelar,
        )
        .map_err(|e: Erro| e.mensagem())?;
        let entrada = Entrada { pacote_id: pacote.ficha.id.clone(), versao: pacote.ficha.versao.clone(), saida: resultado.saida.clone(), data: hoje() };
        Ok((resultado, entrada))
    });
    let resultado = tarefa.await.map_err(|e| e.to_string()).and_then(|r| r).map(|(resultado, entrada)| {
        let _ = estado.banco.registrar_traducao(&entrada);
        resultado
    });
    traduzindo.store(false, Ordering::SeqCst);
    if fechar_ao_terminar.load(Ordering::SeqCst) {
        app_fim.exit(0);
    }
    resultado
}

#[tauri::command]
pub fn definir_mostrar_trailers(estado: State<'_, Estado>, mostrar: bool) -> Result<ListaJogos, String> {
    estado.definir_trailers(mostrar)?;
    Ok(estado.lista())
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
    use comum::servidor::{Resposta, Servidor};
    use std::fs;

    const CNF: &[u8] = b"BOOT2 = cdrom0:\\SCUS_972.65;1\r\n";
    const HASH: &str = "f51c94dd47cc2eb385d5733389428121";
    const HOJE: &str = "2026-11-05";

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

    /// Catálogo de teste com o Jak II publicado (mesmo id do rascunho).
    fn catalogo(cache: &Path) -> (Servidor, Sincronizacao) {
        let s = Servidor::novo();
        let fichas = vec![comum::ficha_catalogo("jak2-usa-ptbr", "1.0", &s.url(""), HASH, 100, b"PATCH", None)];
        s.definir("/indice.json", Resposta::Ok(comum::indice(&fichas)));
        let sinc = sincronizar_interno(cache, &s.url("/indice.json"));
        (s, sinc)
    }

    #[test]
    fn lista_jogos_de_pasta_de_rascunhos() {
        let raiz = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        pasta_com_pacote(raiz.path(), HASH);
        let lista = montar_lista(cache.path(), Some(raiz.path()), &Historico::default(), &sincronizacao_inicial(cache.path()), HOJE);
        assert_eq!(lista.jogos.len(), 1);
        assert_eq!(lista.jogos[0].nome, "Jak II");
        let t = &lista.jogos[0].traducoes[0];
        assert_eq!(t.regiao, "USA");
        assert_eq!(t.id, "rascunho:jak2-usa-ptbr");
        assert_eq!(t.origem, "rascunho");
        assert!(t.patch_disponivel);
        assert!(lista.jogos[0].capa.as_deref().unwrap().starts_with("data:image/png;base64,"));
        assert_eq!(lista.pasta_rascunhos.as_deref(), Some(raiz.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn lista_junta_catalogo_e_rascunhos() {
        let raiz = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        let (_s, sinc) = catalogo(cache.path());
        assert!(!sinc.offline, "{:?}", sinc.aviso);
        let p = raiz.path().join("outro");
        pasta_com_pacote(&p, HASH);
        let ficha = p.join("jak2").join("ficha.json");
        let texto = fs::read_to_string(&ficha)
            .unwrap()
            .replace(r#""id":"jak2","nome":"Jak II""#, r#""id":"sly","nome":"Sly Cooper""#)
            .replace("jak2-usa-ptbr", "sly-ptbr");
        fs::write(&ficha, texto).unwrap();
        let lista = montar_lista(cache.path(), Some(raiz.path()), &Historico::default(), &sinc, HOJE);
        let nomes: Vec<&str> = lista.jogos.iter().map(|j| j.nome.as_str()).collect();
        assert_eq!(nomes, vec!["Jak II", "Sly Cooper"]);
        let jak = &lista.jogos[0].traducoes[0];
        assert_eq!(jak.origem, "catalogo");
        assert!(jak.novo);
        assert_eq!(jak.tamanho_download, Some(5));
    }

    #[test]
    fn rascunho_com_mesmo_id_aparece_separado() {
        let raiz = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        let (_s, sinc) = catalogo(cache.path());
        pasta_com_pacote(raiz.path(), HASH);
        let lista = montar_lista(cache.path(), Some(raiz.path()), &Historico::default(), &sinc, HOJE);
        assert_eq!(lista.jogos.len(), 1);
        let ids: Vec<&str> = lista.jogos[0].traducoes.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, vec!["jak2-usa-ptbr", "rascunho:jak2-usa-ptbr"]);
        assert!(achar_pacote(cache.path(), Some(raiz.path()), "rascunho:jak2-usa-ptbr").unwrap().patch_disponivel());
        assert!(!achar_pacote(cache.path(), Some(raiz.path()), "jak2-usa-ptbr").unwrap().patch_disponivel());
    }

    #[test]
    fn lista_sem_rede_usa_o_cache_e_avisa() {
        let cache = tempfile::tempdir().unwrap();
        catalogo(cache.path());
        let sinc = sincronizar_interno(cache.path(), "http://127.0.0.1:9/indice.json");
        let lista = montar_lista(cache.path(), None, &Historico::default(), &sinc, HOJE);
        assert!(lista.catalogo.offline);
        assert!(lista.catalogo.aviso.as_deref().unwrap().starts_with("Sem conexão"));
        assert_eq!(lista.jogos.len(), 1);
    }

    #[test]
    fn lista_leva_o_trailer_e_a_preferencia_de_video() {
        let cache = tempfile::tempdir().unwrap();
        let s = Servidor::novo();
        let ficha = comum::ficha_catalogo("jak2-usa-ptbr", "1.0", &s.url(""), HASH, 100, b"PATCH", None)
            .replace(r#""arquivos":"#, r#""midia":{"trailer_youtube":"Pj-ueiUYEbg"},"arquivos":"#);
        s.definir("/indice.json", Resposta::Ok(comum::indice(&[ficha])));
        let sinc = sincronizar_interno(cache.path(), &s.url("/indice.json"));
        let mut lista = montar_lista(cache.path(), None, &Historico::default(), &sinc, HOJE);
        assert_eq!(lista.jogos[0].traducoes[0].trailer_youtube.as_deref(), Some("Pj-ueiUYEbg"));
        assert!(lista.mostrar_trailers, "vídeo ligado por padrão");

        let dados = tempfile::tempdir().unwrap();
        let estado = Estado::novo(dados.path().to_path_buf());
        estado.definir_trailers(false).unwrap();
        lista = Estado::novo(dados.path().to_path_buf()).lista();
        assert!(!lista.mostrar_trailers, "a escolha fica salva");
    }

    #[test]
    fn traducao_sem_patch_baixado_informa_patch_disponivel_falso() {
        let cache = tempfile::tempdir().unwrap();
        let (_s, sinc) = catalogo(cache.path());
        let lista = montar_lista(cache.path(), None, &Historico::default(), &sinc, HOJE);
        assert!(!lista.jogos[0].traducoes[0].patch_disponivel);
    }

    /// Os links dos créditos abrem no navegador pelo plugin opener; sem esta permissão o clique não faz nada.
    #[test]
    fn sincronizacao_nao_roda_em_dobro_nem_durante_traducao() {
        let dados = tempfile::tempdir().unwrap();
        let estado = Estado::novo(dados.path().to_path_buf());
        assert!(estado.comecar_sincronizacao(), "a primeira começa");
        assert!(!estado.comecar_sincronizacao(), "a segunda espera a primeira acabar");
        estado.terminar_sincronizacao();
        estado.traduzindo.store(true, Ordering::SeqCst);
        assert!(!estado.comecar_sincronizacao(), "durante uma tradução não sincroniza");
        estado.traduzindo.store(false, Ordering::SeqCst);
        assert!(estado.comecar_sincronizacao());
    }

    #[test]
    fn traducao_nao_comeca_durante_sincronizacao() {
        let dados = tempfile::tempdir().unwrap();
        let estado = Estado::novo(dados.path().to_path_buf());
        assert!(estado.comecar_sincronizacao());
        assert!(!estado.comecar_traducao(), "espera o catálogo terminar de atualizar");
        estado.terminar_sincronizacao();
        assert!(estado.comecar_traducao());
        assert!(!estado.comecar_traducao(), "uma tradução por vez");
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
    fn sem_pasta_e_sem_catalogo_a_lista_fica_vazia() {
        let cache = tempfile::tempdir().unwrap();
        let lista = montar_lista(cache.path(), None, &Historico::default(), &sincronizacao_inicial(cache.path()), HOJE);
        assert!(lista.jogos.is_empty());
        assert!(lista.pasta_rascunhos.is_none());
    }

    #[test]
    fn conferir_iso_de_outro_jogo_devolve_mensagem() {
        let raiz = tempfile::tempdir().unwrap();
        pasta_com_pacote(raiz.path(), HASH);
        let outra = comum::gravar(
            raiz.path(),
            "outra.iso",
            &comum::iso_sintetica(&[("SYSTEM.CNF", b"BOOT2 = cdrom0:\\SLES_123.45;1\r\n"), ("SLES_123.45", b"ELF")]),
        );
        let pacote = nucleo::carregar_pacotes(raiz.path()).pacotes.remove(0);
        let c = conferir_interno(&pacote, &outra);
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
        let pacote = nucleo::carregar_pacotes(&raiz.path().join("pacotes")).pacotes.remove(0);
        let c = conferir_interno(&pacote, &caminho);
        assert!(c.serve, "{:?}", c.mensagem);
        assert_eq!(c.saida_sugerida, raiz.path().join("Jak II (USA) PT-BR v1.0.iso").to_string_lossy());
        assert!(!c.ja_existe);
    }

    #[test]
    fn sem_xdelta3_a_mensagem_explica() {
        let pasta = tempfile::tempdir().unwrap();
        assert_eq!(
            xdelta3_na_pasta(pasta.path()).unwrap_err(),
            "O componente que aplica a tradução (xdelta3) não foi encontrado. Reinstale o Tradumix."
        );
        fs::write(pasta.path().join("xdelta3.exe"), b"MZ").unwrap();
        assert_eq!(xdelta3_na_pasta(pasta.path()).unwrap(), pasta.path().join("xdelta3.exe"));
    }

    #[test]
    fn app_ignora_a_pasta_de_rascunhos_de_um_config_antigo() {
        let dados = tempfile::tempdir().unwrap();
        let rascunhos = tempfile::tempdir().unwrap();
        pasta_com_pacote(rascunhos.path(), HASH);
        let config = serde_json::json!({ "pasta_rascunhos": rascunhos.path() });
        fs::write(dados.path().join("config.json"), config.to_string()).unwrap();
        let estado = Estado::novo(dados.path().to_path_buf());
        let lista = estado.lista();
        assert!(lista.jogos.is_empty(), "{:?}", lista.jogos.iter().map(|j| &j.nome).collect::<Vec<_>>());
        assert!(lista.invalidos.is_empty());
    }

    /// Catálogo com duas traduções novas (dentro dos 14 dias), uma de cada jogo.
    fn lista_com_dois_novos(cache: &Path) -> (Servidor, ListaJogos) {
        let s = Servidor::novo();
        let jak = comum::ficha_catalogo("jak2-usa-ptbr", "1.0", &s.url(""), HASH, 100, b"PATCH", None);
        let sly = jak.replace(r#""id":"jak2","nome":"Jak II""#, r#""id":"sly","nome":"Sly Cooper""#).replace("jak2-usa-ptbr", "sly-ptbr");
        s.definir("/indice.json", Resposta::Ok(comum::indice(&[jak, sly])));
        let sinc = sincronizar_interno(cache, &s.url("/indice.json"));
        let lista = montar_lista(cache, None, &Historico::default(), &sinc, HOJE);
        assert!(lista.jogos.iter().all(|j| j.traducoes[0].novo), "as duas estão dentro dos 14 dias");
        (s, lista)
    }

    fn novos(lista: &ListaJogos) -> Vec<&str> {
        lista.jogos.iter().flat_map(|j| &j.traducoes).filter(|t| t.novo).map(|t| t.id.as_str()).collect()
    }

    #[test]
    fn novo_so_na_primeira_vez_que_aparece() {
        let cache = tempfile::tempdir().unwrap();
        let dados = tempfile::tempdir().unwrap();
        let (_s, lista) = lista_com_dois_novos(cache.path());

        // Primeira abertura: aparece como novo, e continua assim enquanto o app estiver aberto.
        let banco = Banco::abrir(dados.path()).unwrap();
        let vistos = Vistos::carregar(&banco);
        let mut primeira = lista.clone();
        vistos.marcar(&mut primeira, &banco);
        assert_eq!(novos(&primeira), vec!["jak2-usa-ptbr", "sly-ptbr"]);
        let mut de_novo = lista.clone();
        vistos.marcar(&mut de_novo, &banco);
        assert_eq!(novos(&de_novo).len(), 2, "na mesma abertura o selo não some de repente");
        drop(banco);

        // Próxima abertura do app: já foi visto.
        let banco = Banco::abrir(dados.path()).unwrap();
        let mut depois = lista.clone();
        Vistos::carregar(&banco).marcar(&mut depois, &banco);
        assert!(novos(&depois).is_empty());
    }

    #[test]
    fn jogo_que_chega_depois_aparece_como_novo() {
        let cache = tempfile::tempdir().unwrap();
        let dados = tempfile::tempdir().unwrap();
        let (_s, lista) = lista_com_dois_novos(cache.path());
        let banco = Banco::abrir(dados.path()).unwrap();
        banco.marcar_vistos(&["jak2-usa-ptbr".to_string()]).unwrap();
        let mut agora = lista.clone();
        Vistos::carregar(&banco).marcar(&mut agora, &banco);
        assert_eq!(novos(&agora), vec!["sly-ptbr"]);
        let mut depois = lista.clone();
        Vistos::carregar(&banco).marcar(&mut depois, &banco);
        assert!(novos(&depois).is_empty());
    }

    #[test]
    fn novo_some_pelo_estado_do_app_na_abertura_seguinte() {
        let cache = tempfile::tempdir().unwrap();
        let (_s, lista) = lista_com_dois_novos(cache.path());
        let dados = tempfile::tempdir().unwrap();
        let banco = Banco::abrir(dados.path()).unwrap();
        let mut primeira = lista.clone();
        Vistos::carregar(&banco).marcar(&mut primeira, &banco);
        drop(banco);
        // O Estado de uma nova abertura lê os vistos do mesmo banco.
        let estado = Estado::novo(dados.path().to_path_buf());
        let mut segunda = lista.clone();
        estado.vistos.marcar(&mut segunda, &estado.banco);
        assert!(novos(&segunda).is_empty());
    }
}
