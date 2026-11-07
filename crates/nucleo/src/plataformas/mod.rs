//! As plataformas que esta versão do app sabe instalar. Cada uma diz como reconhecer a cópia
//! original da pessoa, como chamar a cópia traduzida e de onde vem a capa.
//!
//! Para acrescentar uma plataforma: criar o módulo dela, uma variante em `Plataforma` e em `Alvo`,
//! e pôr a plataforma em `PLATAFORMAS`. Traduções de plataformas que não estão aqui ficam de fora.

pub mod ps2;

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{Erro, Ficha};

pub use ps2::{OrigemPs2, ResultadoPs2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plataforma {
    Ps2,
}

/// Todas as plataformas conhecidas, na ordem em que aparecem no app.
pub const PLATAFORMAS: &[Plataforma] = &[Plataforma::Ps2];

/// O que a interface precisa saber de uma plataforma.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InfoPlataforma {
    pub id: &'static str,
    /// Nome para mostrar ("PlayStation 2").
    pub nome: &'static str,
    /// Nome curto, usado no card e no filtro ("PS2").
    pub sigla: &'static str,
    /// Como se chama o arquivo do jogo que a pessoa escolhe ("ISO").
    pub arquivo: &'static str,
    /// Extensões aceitas no seletor de arquivo.
    pub extensoes: &'static [&'static str],
    /// A plataforma tem RetroAchievements (o app confere o hash no fim).
    pub tem_ra: bool,
}

impl Plataforma {
    pub fn de_id(id: &str) -> Option<Plataforma> {
        PLATAFORMAS.iter().copied().find(|p| p.info().id == id)
    }

    pub fn info(&self) -> InfoPlataforma {
        match self {
            Plataforma::Ps2 => ps2::INFO,
        }
    }

    /// Posição na lista central, para ordenar.
    pub fn ordem(&self) -> usize {
        PLATAFORMAS.iter().position(|p| p == self).unwrap_or(usize::MAX)
    }
}

/// O que a tradução espera encontrar e o que ela gera, já lido pela peça da plataforma.
#[derive(Debug, Clone, PartialEq)]
pub enum Alvo {
    Ps2 { origem: OrigemPs2, resultado: ResultadoPs2 },
}

/// Lê `origem` e `resultado` da ficha conforme a plataforma dela.
pub fn ler_alvo(ficha: &Ficha) -> Result<Alvo, String> {
    match Plataforma::de_id(&ficha.plataforma) {
        Some(Plataforma::Ps2) => ps2::ler(ficha),
        None => Err(format!("plataforma desconhecida: {}", ficha.plataforma)),
    }
}

impl Alvo {
    pub fn plataforma(&self) -> Plataforma {
        match self {
            Alvo::Ps2 { .. } => Plataforma::Ps2,
        }
    }

    /// Confere, sem ler tudo, se o arquivo escolhido é a cópia que a tradução espera.
    pub fn conferir(&self, ficha: &Ficha, arquivo: &Path) -> Result<(), Erro> {
        match self {
            Alvo::Ps2 { origem, .. } => ps2::conferir(ficha, origem, arquivo),
        }
    }

    /// Nome sugerido para a cópia traduzida, na mesma pasta da original.
    pub fn nome_saida(&self, ficha: &Ficha, original: &Path) -> PathBuf {
        match self {
            Alvo::Ps2 { origem, .. } => ps2::nome_saida(ficha, origem, original),
        }
    }

    /// Tamanho esperado da cópia traduzida, em bytes.
    pub fn tamanho_resultado(&self) -> u64 {
        match self {
            Alvo::Ps2 { resultado, .. } => resultado.tamanho,
        }
    }

    /// Hash do RetroAchievements que a cópia original tem (`None` em plataforma sem RA).
    pub fn hash_ra_esperado(&self) -> Option<&str> {
        match self {
            Alvo::Ps2 { origem, .. } => Some(&origem.hash_ra),
        }
    }

    /// Hash do RetroAchievements do arquivo gerado (`None` em plataforma sem RA).
    pub fn verificar_resultado(&self, gerado: &Path) -> Result<Option<String>, Erro> {
        match self {
            Alvo::Ps2 { .. } => Ok(Some(crate::identificar(gerado)?.hash_ra)),
        }
    }

    /// Endereço da capa no acervo da plataforma; `modelo` troca o acervo padrão (testes).
    pub fn url_capa(&self, modelo: Option<&str>) -> Option<String> {
        match self {
            Alvo::Ps2 { origem, .. } => Some(modelo.unwrap_or(ps2::ACERVO_CAPAS).replace("{serial}", &origem.serial)),
        }
    }

    /// Para qual versão do jogo é a tradução, como texto pronto ("versão USA (SCUS-97265)").
    pub fn detalhes(&self) -> String {
        match self {
            Alvo::Ps2 { origem, .. } => format!("versão {} ({})", origem.regiao.nome(), origem.serial),
        }
    }

    /// O que a pessoa precisa escolher para traduzir.
    pub fn instrucao(&self) -> String {
        match self {
            Alvo::Ps2 { origem, .. } => {
                format!("Use a sua ISO original da versão {}. Ela não é alterada: a traduzida é gravada ao lado.", origem.regiao.nome())
            }
        }
    }
}

/// Tira o que o Windows não aceita num nome de arquivo (e qualquer separador de pasta):
/// `:` vira ` -` ("Jogo: Subtítulo" → "Jogo - Subtítulo"), o resto some.
pub(crate) fn nome_de_arquivo_seguro(nome: &str) -> String {
    let mut limpo = String::with_capacity(nome.len());
    for c in nome.chars() {
        match c {
            ':' => limpo.push_str(" -"),
            '<' | '>' | '"' | '/' | '\\' | '|' | '?' | '*' => limpo.push(' '),
            c if c.is_control() => {}
            c => limpo.push(c),
        }
    }
    let junto = limpo.split_whitespace().collect::<Vec<_>>().join(" ");
    let aparado = junto.trim_matches(|c: char| c == '.' || c == ' ').to_string();
    if aparado.is_empty() { "Jogo".into() } else { aparado }
}
