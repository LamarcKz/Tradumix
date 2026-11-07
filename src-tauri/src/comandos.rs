//! Comandos que a interface chama. Cada um é uma função interna testável mais um envoltório Tauri.
//!
//! O app mostra só as traduções do catálogo (sincronizado para `<dados>/catalogo`), agrupadas por
//! jogo e, dentro do jogo, por plataforma.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use base64::Engine;
use nucleo::{
    Atualizacao, Capas, Entrada, Erro, Estado as EstadoTraducao, Historico, Imagem, InfoPlataforma, Pacote, Pedido, RedeUreq, Resultado, Sincronizacao, Tradutor,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::banco::Banco;

const URL_CATALOGO: &str = "https://raw.githubusercontent.com/LamarcKz/Tradumix-catalogo/main/indice-v2.json";

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
        for t in lista.jogos.iter_mut().flat_map(|j| j.plataformas.iter_mut()).flat_map(|p| p.traducoes.iter_mut()) {
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
        let mut lista = montar_lista(&self.cache(), &historico, &self.sincronizacao(), &hoje());
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
    pub versao: String,
    pub estado: EstadoTraducao,
    pub idioma: String,
    /// Para qual versão do jogo é a tradução, já como texto ("versão USA (SCUS-97265)").
    pub detalhes: String,
    /// O que a pessoa precisa escolher para traduzir.
    pub instrucao: String,
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

/// As traduções de um jogo numa plataforma.
#[derive(Debug, Clone, Serialize)]
pub struct PlataformaDoJogoDto {
    pub info: InfoPlataforma,
    pub capa: Option<String>,
    pub traducoes: Vec<TraducaoDto>,
}

/// Um card: o jogo com as plataformas em que há tradução (na ordem da lista central do núcleo).
#[derive(Debug, Clone, Serialize)]
pub struct JogoDto {
    pub id: String,
    pub nome: String,
    pub plataformas: Vec<PlataformaDoJogoDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListaJogos {
    pub jogos: Vec<JogoDto>,
    pub atualizacoes: Vec<Atualizacao>,
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
    nucleo::sincronizar(&RedeUreq::novo(), url, cache, local, Capas::Acervos)
}

pub fn achar_pacote(cache: &Path, id: &str) -> Option<Pacote> {
    nucleo::carregar_pacotes(cache).pacotes.into_iter().find(|p| p.ficha.id == id)
}

fn traducao_dto(p: &Pacote, hoje: &str) -> TraducaoDto {
    let f = &p.ficha;
    let disponivel = p.patch_disponivel();
    TraducaoDto {
        id: f.id.clone(),
        versao: f.versao.clone(),
        estado: f.estado,
        idioma: f.idioma.clone(),
        detalhes: p.alvo.detalhes(),
        instrucao: p.alvo.instrucao(),
        tradutores: f.tradutores.clone(),
        permissao: f.permissao.clone(),
        descricao: f.descricao.clone(),
        traduzido: f.traduzido.clone(),
        limitacoes: f.limitacoes.clone(),
        data: f.data.clone(),
        fundo: p.imagem(Imagem::Fundo).and_then(|c| data_url(&c)),
        novo: nucleo::e_novo(f, hoje),
        patch_disponivel: disponivel,
        tamanho_download: if disponivel { None } else { Some(f.patch().tamanho) },
        trailer_youtube: f.midia.as_ref().and_then(|m| m.trailer_youtube.clone()),
    }
}

/// Fichas do cache que não abrem (por exemplo, de uma versão antiga do app) ficam de fora; a próxima
/// sincronização refaz o cache.
pub fn montar_lista(cache: &Path, historico: &Historico, sinc: &Sincronizacao, hoje: &str) -> ListaJogos {
    let pacotes = nucleo::carregar_pacotes(cache).pacotes;
    let mut jogos: Vec<JogoDto> = Vec::new();
    for p in &pacotes {
        let indice_jogo = match jogos.iter().position(|j| j.id == p.ficha.jogo.id) {
            Some(i) => i,
            None => {
                jogos.push(JogoDto { id: p.ficha.jogo.id.clone(), nome: p.ficha.jogo.nome.clone(), plataformas: Vec::new() });
                jogos.len() - 1
            }
        };
        let jogo = &mut jogos[indice_jogo];
        let info = p.alvo.plataforma().info();
        let plataforma = match jogo.plataformas.iter().position(|x| x.info.id == info.id) {
            Some(i) => &mut jogo.plataformas[i],
            None => {
                jogo.plataformas.push(PlataformaDoJogoDto { info, capa: None, traducoes: Vec::new() });
                jogo.plataformas.last_mut().unwrap()
            }
        };
        if plataforma.capa.is_none() {
            plataforma.capa = p.imagem(Imagem::Capa).and_then(|c| data_url(&c));
        }
        plataforma.traducoes.push(traducao_dto(p, hoje));
    }
    let ordem = |id: &str| nucleo::Plataforma::de_id(id).map(|p| p.ordem()).unwrap_or(usize::MAX);
    for jogo in &mut jogos {
        jogo.plataformas.sort_by_key(|p| ordem(p.info.id));
    }
    jogos.sort_by(|a, b| a.nome.to_lowercase().cmp(&b.nome.to_lowercase()));
    ListaJogos { jogos, atualizacoes: historico.atualizacoes(&pacotes), catalogo: sinc.clone(), mostrar_trailers: true }
}

pub fn conferir_interno(pacote: &Pacote, iso: &Path) -> ConferenciaDto {
    let saida = nucleo::nome_saida(pacote, iso);
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
    let pacote = achar_pacote(&estado.cache(), &pacote_id).ok_or("Tradução não encontrada.")?;
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
        let pacote = achar_pacote(&cache, &pacote_id).ok_or("Tradução não encontrada.")?;
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

    /// Pacote de PS2 numa pasta, com o patch já baixado e capa própria em PNG.
    fn pasta_com_pacote_de_tamanho(raiz: &Path, hash: &str, tamanho: u64) {
        let p = raiz.join("jak2");
        fs::create_dir_all(&p).unwrap();
        fs::write(p.join("patch.xdelta"), b"VCD").unwrap();
        fs::write(p.join("capa.png"), [0x89, b'P', b'N', b'G']).unwrap();
        let sha = comum::sha256_hex(b"VCD");
        let imagens = format!(r#","imagens":{{"capa":{{"nome":"capa.png","url":"https://x/capa.png","sha256":"{sha}","tamanho":4}}}}"#);
        let ficha = comum::ficha_ps2("jak2-usa-ptbr", "1.0", hash, tamanho, tamanho, "https://x/patch", &sha, 3, &imagens);
        fs::write(p.join("ficha.json"), ficha).unwrap();
    }

    /// Catálogo de teste com o Jak II publicado.
    fn catalogo(cache: &Path) -> (Servidor, Sincronizacao) {
        let s = Servidor::novo();
        let fichas = vec![comum::ficha_catalogo("jak2-usa-ptbr", "1.0", &s.url(""), HASH, 100, b"PATCH", None)];
        s.definir("/indice.json", Resposta::Ok(comum::indice(&fichas)));
        let sinc = sincronizar_interno(cache, &s.url("/indice.json"));
        (s, sinc)
    }

    /// Acrescenta campos no fim de uma ficha em JSON.
    fn com_extra(f: &str, extra: &str) -> String {
        let mut f = f.to_string();
        f.pop();
        format!("{f}{extra}}}")
    }

    #[test]
    fn lista_do_catalogo_por_plataforma() {
        let cache = tempfile::tempdir().unwrap();
        let (_s, sinc) = catalogo(cache.path());
        assert!(!sinc.offline, "{:?}", sinc.aviso);
        let lista = montar_lista(cache.path(), &Historico::default(), &sinc, HOJE);
        assert_eq!(lista.jogos.len(), 1);
        let jogo = &lista.jogos[0];
        assert_eq!(jogo.nome, "Jak II");
        assert_eq!(jogo.plataformas.len(), 1);
        assert_eq!(jogo.plataformas[0].info.sigla, "PS2");
        let t = &jogo.plataformas[0].traducoes[0];
        assert_eq!(t.id, "jak2-usa-ptbr");
        assert!(t.novo);
        assert_eq!(t.tamanho_download, Some(5));
    }

    #[test]
    fn duas_traducoes_do_mesmo_jogo_ficam_num_jogo_e_numa_plataforma() {
        let cache = tempfile::tempdir().unwrap();
        let s = Servidor::novo();
        let a = comum::ficha_catalogo("jak2-a", "1.0", &s.url(""), HASH, 100, b"PATCH", None);
        let b = comum::ficha_catalogo("jak2-b", "1.0", &s.url(""), HASH, 100, b"PATCH", None);
        s.definir("/indice.json", Resposta::Ok(comum::indice(&[a, b])));
        let sinc = sincronizar_interno(cache.path(), &s.url("/indice.json"));
        let lista = montar_lista(cache.path(), &Historico::default(), &sinc, HOJE);
        assert_eq!(lista.jogos.len(), 1);
        assert_eq!(lista.jogos[0].plataformas.len(), 1);
        assert_eq!(lista.jogos[0].plataformas[0].traducoes.len(), 2);
        assert_eq!(lista.jogos[0].plataformas[0].info.sigla, "PS2");
    }

    #[test]
    fn traducao_leva_detalhes_e_instrucao_da_plataforma() {
        let cache = tempfile::tempdir().unwrap();
        let (_s, sinc) = catalogo(cache.path());
        let lista = montar_lista(cache.path(), &Historico::default(), &sinc, HOJE);
        let t = &lista.jogos[0].plataformas[0].traducoes[0];
        assert_eq!(t.detalhes, "versão USA (SCUS-97265)");
        assert!(t.instrucao.starts_with("Use a sua ISO original da versão USA"), "{}", t.instrucao);
    }

    #[test]
    fn ficha_formato_1_no_cache_nao_aparece() {
        let cache = tempfile::tempdir().unwrap();
        let velho = cache.path().join("jak2-usa-ptbr");
        fs::create_dir_all(&velho).unwrap();
        fs::write(velho.join("ficha.json"), r#"{"formato":1,"id":"jak2-usa-ptbr","jogo":{"id":"jak2","nome":"Jak II","console":"ps2"}}"#).unwrap();
        let lista = montar_lista(cache.path(), &Historico::default(), &sincronizacao_inicial(cache.path()), HOJE);
        assert!(lista.jogos.is_empty());
    }

    #[test]
    fn capa_propria_vai_na_plataforma() {
        let cache = tempfile::tempdir().unwrap();
        pasta_com_pacote(cache.path(), HASH);
        let lista = montar_lista(cache.path(), &Historico::default(), &sincronizacao_inicial(cache.path()), HOJE);
        let plataforma = &lista.jogos[0].plataformas[0];
        assert!(plataforma.capa.as_deref().unwrap().starts_with("data:image/png;base64,"));
        assert!(plataforma.traducoes[0].patch_disponivel);
    }

    #[test]
    fn lista_sem_rede_usa_o_cache_e_avisa() {
        let cache = tempfile::tempdir().unwrap();
        catalogo(cache.path());
        let sinc = sincronizar_interno(cache.path(), "http://127.0.0.1:9/indice.json");
        let lista = montar_lista(cache.path(), &Historico::default(), &sinc, HOJE);
        assert!(lista.catalogo.offline);
        assert!(lista.catalogo.aviso.as_deref().unwrap().starts_with("Sem conexão"));
        assert_eq!(lista.jogos.len(), 1);
    }

    #[test]
    fn lista_leva_o_trailer_e_a_preferencia_de_video() {
        let cache = tempfile::tempdir().unwrap();
        let s = Servidor::novo();
        let ficha = com_extra(
            &comum::ficha_catalogo("jak2-usa-ptbr", "1.0", &s.url(""), HASH, 100, b"PATCH", None),
            r#","midia":{"trailer_youtube":"Pj-ueiUYEbg"}"#,
        );
        s.definir("/indice.json", Resposta::Ok(comum::indice(&[ficha])));
        let sinc = sincronizar_interno(cache.path(), &s.url("/indice.json"));
        let mut lista = montar_lista(cache.path(), &Historico::default(), &sinc, HOJE);
        assert_eq!(lista.jogos[0].plataformas[0].traducoes[0].trailer_youtube.as_deref(), Some("Pj-ueiUYEbg"));
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
        let lista = montar_lista(cache.path(), &Historico::default(), &sinc, HOJE);
        assert!(!lista.jogos[0].plataformas[0].traducoes[0].patch_disponivel);
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
    fn sem_catalogo_a_lista_fica_vazia() {
        let cache = tempfile::tempdir().unwrap();
        let lista = montar_lista(cache.path(), &Historico::default(), &sincronizacao_inicial(cache.path()), HOJE);
        assert!(lista.jogos.is_empty());
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

    /// Catálogo com duas traduções novas (dentro dos 14 dias), uma de cada jogo.
    fn lista_com_dois_novos(cache: &Path) -> (Servidor, ListaJogos) {
        let s = Servidor::novo();
        let jak = comum::ficha_catalogo("jak2-usa-ptbr", "1.0", &s.url(""), HASH, 100, b"PATCH", None);
        let sly = jak.replace(r#""id":"jak2","nome":"Jak II""#, r#""id":"sly","nome":"Sly Cooper""#).replace("jak2-usa-ptbr", "sly-ptbr");
        s.definir("/indice.json", Resposta::Ok(comum::indice(&[jak, sly])));
        let sinc = sincronizar_interno(cache, &s.url("/indice.json"));
        let lista = montar_lista(cache, &Historico::default(), &sinc, HOJE);
        assert!(lista.jogos.iter().all(|j| j.plataformas[0].traducoes[0].novo), "as duas estão dentro dos 14 dias");
        (s, lista)
    }

    fn novos(lista: &ListaJogos) -> Vec<&str> {
        lista.jogos.iter().flat_map(|j| &j.plataformas).flat_map(|p| &p.traducoes).filter(|t| t.novo).map(|t| t.id.as_str()).collect()
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
