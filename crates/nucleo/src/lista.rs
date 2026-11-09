//! A lista que as telas recebem: um jogo por card, com as plataformas e as traduções de cada uma.
//! Fica no núcleo para o app e o `tradumix-cli` (prévia) gerarem exatamente os mesmos dados.

use std::path::Path;

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::{Atualizacao, Estado, Imagem, InfoPlataforma, Pacote, Sincronizacao, Tradutor, e_novo};

/// Selo do RetroAchievements: a cópia traduzida continua com o hash da original ou não.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SituacaoRa {
    Compativel,
    NaoReconhece,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraducaoDto {
    pub id: String,
    pub versao: String,
    pub estado: Estado,
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
    /// Tamanho do que a instalação baixa, quando ainda não está no PC.
    pub tamanho_download: Option<u64>,
    /// Id do trailer no YouTube.
    pub trailer_youtube: Option<String>,
    /// `None` em plataforma sem RetroAchievements.
    pub ra: Option<SituacaoRa>,
    /// A tradução põe na pasta do jogo um arquivo que roda.
    pub contem_programa: bool,
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

/// Imagem da pasta do pacote como `data:` URL (as telas não leem arquivos do disco).
pub fn data_url(caminho: &Path) -> Option<String> {
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
        novo: e_novo(f, hoje),
        patch_disponivel: disponivel,
        tamanho_download: if disponivel { None } else { Some(f.baixavel_principal().tamanho) },
        trailer_youtube: f.midia.as_ref().and_then(|m| m.trailer_youtube.clone()),
        ra: p.alvo.ra_compativel().map(|ok| if ok { SituacaoRa::Compativel } else { SituacaoRa::NaoReconhece }),
        contem_programa: f.contem_programa(),
    }
}

/// Agrupa os pacotes por jogo e, dentro do jogo, por plataforma (na ordem central); jogos por nome.
pub fn montar_jogos(pacotes: &[Pacote], hoje: &str) -> Vec<JogoDto> {
    let mut jogos: Vec<JogoDto> = Vec::new();
    for p in pacotes {
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
    let ordem = |id: &str| crate::Plataforma::de_id(id).map(|p| p.ordem()).unwrap_or(usize::MAX);
    for jogo in &mut jogos {
        jogo.plataformas.sort_by_key(|p| ordem(p.info.id));
    }
    jogos.sort_by(|a, b| a.nome.to_lowercase().cmp(&b.nome.to_lowercase()));
    jogos
}
