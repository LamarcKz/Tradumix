//! As plataformas que esta versão do app sabe instalar. Cada uma diz como reconhecer a cópia
//! original da pessoa, como chamar a cópia traduzida e de onde vem a capa.
//!
//! Para acrescentar uma plataforma: criar o módulo dela, uma variante em `Plataforma` e em `Alvo`,
//! e pôr a plataforma em `PLATAFORMAS`. Traduções de plataformas que não estão aqui ficam de fora.

pub mod pc;
pub mod ps1;
pub mod ps2;

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{Erro, Ficha};

pub use pc::{OrigemPc, ReferenciaPc};
pub use ps1::{OrigemPs1, ResultadoPs1};
pub use ps2::{OrigemPs2, ResultadoPs2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plataforma {
    Ps2,
    Ps1,
    Pc,
}

/// Todas as plataformas conhecidas, na ordem em que aparecem no app.
pub const PLATAFORMAS: &[Plataforma] = &[Plataforma::Ps2, Plataforma::Ps1, Plataforma::Pc];

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
    /// A pessoa escolhe uma pasta (o jogo instalado) em vez de um arquivo.
    pub escolhe_pasta: bool,
}

impl Plataforma {
    pub fn de_id(id: &str) -> Option<Plataforma> {
        PLATAFORMAS.iter().copied().find(|p| p.info().id == id)
    }

    pub fn info(&self) -> InfoPlataforma {
        match self {
            Plataforma::Ps2 => ps2::INFO,
            Plataforma::Ps1 => ps1::INFO,
            Plataforma::Pc => pc::INFO,
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
    Ps1 { origem: OrigemPs1, resultado: ResultadoPs1 },
    Pc { origem: OrigemPc },
}

/// Lê `origem` e `resultado` da ficha conforme a plataforma dela.
pub fn ler_alvo(ficha: &Ficha) -> Result<Alvo, String> {
    match Plataforma::de_id(&ficha.plataforma) {
        Some(Plataforma::Ps2) => ps2::ler(ficha),
        Some(Plataforma::Ps1) => ps1::ler(ficha),
        Some(Plataforma::Pc) => pc::ler(ficha),
        None => Err(format!("plataforma desconhecida: {}", ficha.plataforma)),
    }
}

/// `origem.hash_ra` e `resultado.hash_ra` com 32 caracteres hexadecimais.
pub(crate) fn conferir_hashes(origem: &str, resultado: &str) -> Result<(), String> {
    for (campo, hash) in [("origem.hash_ra", origem), ("resultado.hash_ra", resultado)] {
        if hash.len() != 32 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("{campo} deve ter 32 caracteres hexadecimais"));
        }
    }
    Ok(())
}

impl Alvo {
    pub fn plataforma(&self) -> Plataforma {
        match self {
            Alvo::Ps2 { .. } => Plataforma::Ps2,
            Alvo::Ps1 { .. } => Plataforma::Ps1,
            Alvo::Pc { .. } => Plataforma::Pc,
        }
    }

    /// Confere, sem ler tudo, se o arquivo escolhido é a cópia que a tradução espera. `Ok(Some)`
    /// traz um aviso que não impede a instalação (PC com versão do jogo diferente da testada).
    pub fn conferir(&self, ficha: &Ficha, arquivo: &Path) -> Result<Option<String>, Erro> {
        match self {
            Alvo::Ps2 { origem, .. } => ps2::conferir(ficha, origem, arquivo).map(|()| None),
            Alvo::Ps1 { origem, .. } => ps1::conferir(ficha, origem, arquivo).map(|()| None),
            Alvo::Pc { origem } => pc::conferir(ficha, origem, arquivo),
        }
    }

    /// Nome sugerido para a cópia traduzida, na mesma pasta da original.
    pub fn nome_saida(&self, ficha: &Ficha, original: &Path) -> PathBuf {
        match self {
            Alvo::Ps2 { origem, .. } => ps2::nome_saida(ficha, origem, original),
            Alvo::Ps1 { origem, .. } => ps1::nome_saida(ficha, origem, original),
            // No PC a tradução vai para a própria pasta do jogo.
            Alvo::Pc { .. } => original.to_path_buf(),
        }
    }

    /// Todos os arquivos que a instalação cria a partir de `saida` (o primeiro é a própria `saida`).
    pub fn arquivos_saida(&self, original: &Path, saida: &Path) -> Result<Vec<PathBuf>, Erro> {
        match self {
            Alvo::Ps2 { .. } => Ok(vec![saida.to_path_buf()]),
            Alvo::Ps1 { .. } => ps1::arquivos_saida(original, saida),
            Alvo::Pc { .. } => Err(sem_imagem()),
        }
    }

    /// O arquivo em que o patch é aplicado (PS2: a ISO; PS1: o `.bin` de dados do `.cue`).
    pub fn entrada_patch(&self, original: &Path) -> Result<PathBuf, Erro> {
        match self {
            Alvo::Ps2 { .. } => Ok(original.to_path_buf()),
            Alvo::Ps1 { .. } => ps1::entrada_patch(original),
            Alvo::Pc { .. } => Err(sem_imagem()),
        }
    }

    /// O arquivo que o patch gera (PS2: a própria saída; PS1: o `.bin` de dados novo).
    pub fn saida_patch(&self, original: &Path, saida: &Path) -> Result<PathBuf, Erro> {
        match self {
            Alvo::Ps2 { .. } => Ok(saida.to_path_buf()),
            Alvo::Ps1 { .. } => ps1::saida_patch(original, saida),
            Alvo::Pc { .. } => Err(sem_imagem()),
        }
    }

    /// Termina a cópia traduzida depois do patch (PS1: copia as faixas de áudio e grava o `.cue`).
    pub fn completar_saida(&self, original: &Path, saida: &Path) -> Result<(), Erro> {
        match self {
            Alvo::Ps2 { .. } => Ok(()),
            Alvo::Ps1 { .. } => ps1::completar_saida(original, saida),
            Alvo::Pc { .. } => Err(sem_imagem()),
        }
    }

    /// Bytes gravados além do resultado do patch (PS1: as faixas de áudio copiadas).
    pub fn tamanho_extra(&self, original: &Path) -> u64 {
        match self {
            Alvo::Ps2 { .. } => 0,
            Alvo::Ps1 { .. } => ps1::tamanho_extra(original),
            Alvo::Pc { .. } => 0,
        }
    }

    /// Tamanho esperado do arquivo que o patch gera, em bytes.
    pub fn tamanho_resultado(&self) -> u64 {
        match self {
            Alvo::Ps2 { resultado, .. } => resultado.tamanho,
            Alvo::Ps1 { resultado, .. } => resultado.tamanho,
            Alvo::Pc { .. } => 0,
        }
    }

    /// Hash do RetroAchievements que a cópia original tem (`None` em plataforma sem RA).
    pub fn hash_ra_esperado(&self) -> Option<&str> {
        match self {
            Alvo::Ps2 { origem, .. } => Some(&origem.hash_ra),
            Alvo::Ps1 { origem, .. } => Some(&origem.hash_ra),
            Alvo::Pc { .. } => None,
        }
    }

    /// Hash do RetroAchievements do arquivo que o patch gerou (`None` em plataforma sem RA).
    pub fn verificar_resultado(&self, original: &Path, gerado: &Path) -> Result<Option<String>, Erro> {
        match self {
            Alvo::Ps2 { .. } => Ok(Some(crate::identificar(gerado)?.hash_ra)),
            Alvo::Ps1 { .. } => ps1::verificar_resultado(original, gerado),
            Alvo::Pc { .. } => Ok(None),
        }
    }

    /// Endereço da capa no acervo da plataforma; `modelo` troca o acervo padrão (testes).
    pub fn url_capa(&self, modelo: Option<&str>) -> Option<String> {
        let (acervo, serial) = match self {
            Alvo::Ps2 { origem, .. } => (ps2::ACERVO_CAPAS, &origem.serial),
            Alvo::Ps1 { origem, .. } => (ps1::ACERVO_CAPAS, &origem.serial),
            // Não há um acervo de capas de PC por serial; a ficha traz a própria capa.
            Alvo::Pc { .. } => return None,
        };
        Some(modelo.unwrap_or(acervo).replace("{serial}", serial))
    }

    /// Para qual versão do jogo é a tradução, como texto pronto ("versão USA (SCUS-97265)").
    pub fn detalhes(&self) -> String {
        match self {
            Alvo::Ps2 { origem, .. } => format!("versão {} ({})", origem.regiao.nome(), origem.serial),
            Alvo::Ps1 { origem, .. } => format!("versão {} ({})", origem.regiao.nome(), origem.serial),
            Alvo::Pc { origem } => format!("versão {} ({})", origem.versao, origem.loja),
        }
    }

    /// O que a pessoa precisa escolher para traduzir.
    pub fn instrucao(&self) -> String {
        match self {
            Alvo::Ps2 { origem, .. } => {
                format!("Use a sua ISO original da versão {}. Ela não é alterada: a traduzida é gravada ao lado.", origem.regiao.nome())
            }
            Alvo::Ps1 { origem, .. } => format!(
                "Use o arquivo .cue da sua cópia original da versão {}, com os .bin na mesma pasta. Eles não são alterados: a cópia traduzida é gravada ao lado.",
                origem.regiao.nome()
            ),
            Alvo::Pc { origem } => format!(
                "Escolha a pasta do jogo instalado ({}). Os arquivos trocados são guardados para você poder remover a tradução depois.",
                origem.loja
            ),
        }
    }
}

/// Erro para as operações de imagem pedidas numa plataforma que instala na pasta do jogo.
fn sem_imagem() -> Erro {
    Erro::Io("esta tradução é instalada na pasta do jogo, não numa imagem".into())
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
