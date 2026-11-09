//! Ficha de tradução (`ficha.json`) e pacotes guardados em pastas.

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::plataformas::{Alvo, ler_alvo};

pub const FORMATO_FICHA: u32 = 2;

/// Nome com que a capa do acervo da plataforma é guardada na pasta do pacote.
pub const NOME_CAPA_ACERVO: &str = "capa.jpg";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Estado {
    Rascunho,
    Beta,
    Publicada,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Jogo {
    pub id: String,
    pub nome: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tradutor {
    pub nome: String,
    #[serde(default)]
    pub link: Option<String>,
}

/// Um arquivo que o app baixa: guardado na pasta do pacote com `nome`, conferido por SHA-256 e tamanho.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArquivoBaixavel {
    pub nome: String,
    pub url: String,
    pub sha256: String,
    pub tamanho: u64,
}

/// Um arquivo que a tradução de PC põe na pasta do jogo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArquivoDoPc {
    /// Caminho relativo à pasta do jogo, com `/`.
    pub caminho: String,
    pub sha256: String,
    pub tamanho: u64,
}

/// Como a tradução é instalada, com o que esse jeito precisa baixar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tipo")]
pub enum Instalacao {
    /// Aplica um `.xdelta` na imagem do jogo e grava a cópia traduzida ao lado da original.
    #[serde(rename = "imagem-xdelta")]
    ImagemXdelta { patch: ArquivoBaixavel },
    /// Extrai um `.zip` na pasta do jogo instalado, guardando os originais que forem trocados.
    #[serde(rename = "pasta-do-jogo")]
    PastaDoJogo { pacote: ArquivoBaixavel, arquivos: Vec<ArquivoDoPc> },
}

/// Extensões de arquivo que contam como programa ("Contém programa").
const EXTENSOES_DE_PROGRAMA: &[&str] = &["exe", "dll", "bat", "cmd", "ps1", "vbs", "js", "py", "msi", "scr", "com"];

/// Capa e fundo próprios da tradução (por exemplo, arte do tradutor).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Imagens {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capa: Option<ArquivoBaixavel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fundo: Option<ArquivoBaixavel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ficha {
    pub formato: u32,
    pub id: String,
    pub jogo: Jogo,
    /// Id da plataforma (`ps2`…). Ver `plataformas`.
    pub plataforma: String,
    pub instalacao: Instalacao,
    pub versao: String,
    pub estado: Estado,
    pub idioma: String,
    pub data: String,
    /// A cópia que a tradução espera; o formato é da plataforma.
    pub origem: serde_json::Value,
    /// A cópia que a tradução gera; o formato é da plataforma.
    pub resultado: serde_json::Value,
    pub tradutores: Vec<Tradutor>,
    #[serde(default)]
    pub permissao: Option<String>,
    #[serde(default)]
    pub descricao: String,
    #[serde(default)]
    pub traduzido: Vec<String>,
    #[serde(default)]
    pub limitacoes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imagens: Option<Imagens>,
    /// Links para mídia que fica em outros lugares (o catálogo não guarda imagem nem vídeo de terceiros).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub midia: Option<Midia>,
}

impl Ficha {
    /// Todos os arquivos que a tradução manda baixar (instalação e imagens).
    pub fn arquivos(&self) -> Vec<&ArquivoBaixavel> {
        let mut lista = vec![self.baixavel_principal()];
        if let Some(imagens) = &self.imagens {
            lista.extend(imagens.capa.iter().chain(imagens.fundo.iter()));
        }
        lista
    }

    /// O que a instalação baixa: o patch (`imagem-xdelta`) ou o pacote (`pasta-do-jogo`).
    pub fn baixavel_principal(&self) -> &ArquivoBaixavel {
        match &self.instalacao {
            Instalacao::ImagemXdelta { patch } => patch,
            Instalacao::PastaDoJogo { pacote, .. } => pacote,
        }
    }

    /// A tradução põe na pasta do jogo algum arquivo que roda (`.exe`, `.dll`, script…).
    pub fn contem_programa(&self) -> bool {
        let Instalacao::PastaDoJogo { arquivos, .. } = &self.instalacao else { return false };
        arquivos.iter().any(|a| {
            let nome = a.caminho.rsplit('/').next().unwrap_or(&a.caminho);
            nome.rsplit_once('.').is_some_and(|(_, ext)| EXTENSOES_DE_PROGRAMA.iter().any(|e| ext.eq_ignore_ascii_case(e)))
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Midia {
    /// Id do vídeo no YouTube (11 caracteres), tocado no player do próprio YouTube.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trailer_youtube: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Imagem {
    Capa,
    Fundo,
}

/// Uma tradução guardada numa pasta do cache: a ficha, o que a plataforma leu dela e a pasta.
#[derive(Debug, Clone, PartialEq)]
pub struct Pacote {
    pub ficha: Ficha,
    pub alvo: Alvo,
    pub pasta: PathBuf,
}

impl Pacote {
    /// Onde fica o arquivo principal baixado (o patch ou o pacote de PC).
    pub fn patch(&self) -> PathBuf {
        self.pasta.join(&self.ficha.baixavel_principal().nome)
    }

    /// O patch já foi baixado para a pasta.
    pub fn patch_disponivel(&self) -> bool {
        self.patch().is_file()
    }

    /// Capa própria da ficha ou, sem ela, a capa do acervo se já estiver na pasta.
    pub fn imagem(&self, qual: Imagem) -> Option<PathBuf> {
        let imagens = self.ficha.imagens.as_ref();
        match qual {
            Imagem::Capa => match imagens.and_then(|i| i.capa.as_ref()) {
                Some(capa) => Some(self.pasta.join(&capa.nome)),
                None => Some(self.pasta.join(NOME_CAPA_ACERVO)).filter(|p| p.is_file()),
            },
            Imagem::Fundo => imagens.and_then(|i| i.fundo.as_ref()).map(|f| self.pasta.join(&f.nome)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PacoteInvalido {
    pub pasta: PathBuf,
    pub motivo: String,
}

#[derive(Debug, Clone, Default)]
pub struct CargaPacotes {
    pub pacotes: Vec<Pacote>,
    pub invalidos: Vec<PacoteInvalido>,
}

/// Até quantos níveis abaixo da pasta escolhida o app procura pacotes
/// (ex.: `Traduções` → `Jak II` → `pacote` → `jak2-usa-ptbr`).
const PROFUNDIDADE_MAXIMA: usize = 4;

/// Carrega cada pasta com um `ficha.json` dentro de `pasta`, procurando também nas subpastas.
/// Não desce dentro de um pacote já encontrado.
pub fn carregar_pacotes(pasta: &Path) -> CargaPacotes {
    let mut carga = CargaPacotes::default();
    let mut pastas = Vec::new();
    procurar_pacotes(pasta, 1, &mut pastas);
    pastas.sort();
    for p in pastas {
        match ler_pacote(&p) {
            Ok(pacote) => carga.pacotes.push(pacote),
            Err(motivo) => carga.invalidos.push(PacoteInvalido { pasta: p, motivo }),
        }
    }
    carga
}

fn procurar_pacotes(pasta: &Path, nivel: usize, achados: &mut Vec<PathBuf>) {
    if nivel > PROFUNDIDADE_MAXIMA {
        return;
    }
    let Ok(entradas) = fs::read_dir(pasta) else { return };
    for entrada in entradas.filter_map(|e| e.ok()) {
        let caminho = entrada.path();
        let oculta = entrada.file_name().to_string_lossy().starts_with('.');
        if oculta || !entrada.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        if caminho.join("ficha.json").is_file() {
            achados.push(caminho);
        } else {
            procurar_pacotes(&caminho, nivel + 1, achados);
        }
    }
}

fn ler_pacote(pasta: &Path) -> Result<Pacote, String> {
    let texto = fs::read_to_string(pasta.join("ficha.json")).map_err(|e| format!("não foi possível ler o ficha.json: {e}"))?;
    let ficha: Ficha = serde_json::from_str(&texto).map_err(|e| format!("ficha.json inválido: {e}"))?;
    let alvo = validar_ficha(&ficha)?;
    Ok(Pacote { ficha, alvo, pasta: pasta.to_path_buf() })
}

/// Regras que valem para toda ficha. Devolve o que a plataforma leu de `origem` e `resultado`.
pub(crate) fn validar_ficha(ficha: &Ficha) -> Result<Alvo, String> {
    if ficha.formato != FORMATO_FICHA {
        return Err(format!("formato de ficha {} não suportado (esperado {FORMATO_FICHA})", ficha.formato));
    }
    for (campo, valor) in [("id", &ficha.id), ("jogo.id", &ficha.jogo.id), ("jogo.nome", &ficha.jogo.nome), ("versao", &ficha.versao)] {
        if valor.trim().is_empty() {
            return Err(format!("o campo {campo} está vazio"));
        }
    }
    // O id vira nome de pasta no cache do catálogo.
    if !nome_simples(&ficha.id) || ficha.id.starts_with('.') {
        return Err(format!("o id {} não pode ter barras, ':' nem começar com ponto", ficha.id));
    }
    let alvo = ler_alvo(ficha)?;
    let pasta_do_jogo = matches!(ficha.instalacao, Instalacao::PastaDoJogo { .. });
    if pasta_do_jogo != alvo.plataforma().info().escolhe_pasta {
        return Err("o tipo de instalação não combina com a plataforma".into());
    }
    if let Some(id) = ficha.midia.as_ref().and_then(|m| m.trailer_youtube.as_ref()) {
        if id.len() != 11 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(format!("trailer_youtube deve ser o id do vídeo (11 caracteres), não {id}"));
        }
    }
    for arquivo in ficha.arquivos() {
        if !nome_simples(&arquivo.nome) || arquivo.nome.eq_ignore_ascii_case("ficha.json") {
            return Err(format!("o arquivo {} precisa estar na própria pasta do pacote e não pode se chamar ficha.json", arquivo.nome));
        }
    }
    Ok(alvo)
}

/// Só um nome de arquivo, sem pastas, unidade ou `..`.
fn nome_simples(nome: &str) -> bool {
    !nome.is_empty() && nome != "." && nome != ".." && !nome.contains(['/', '\\', ':'])
}

/// Dias em que uma tradução recém-publicada aparece com o selo "Novo".
const DIAS_NOVO: i64 = 14;

/// A tradução saiu há menos de 14 dias (`data` e `hoje` no formato AAAA-MM-DD).
pub fn e_novo(ficha: &Ficha, hoje: &str) -> bool {
    let ler = |d: &str| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok();
    match (ler(&ficha.data), ler(hoje)) {
        (Some(data), Some(hoje)) => (0..DIAS_NOVO).contains(&(hoje - data).num_days()),
        _ => false,
    }
}

/// Compara versões por partes numéricas: `1.9 < 1.10`.
pub fn comparar_versoes(a: &str, b: &str) -> Ordering {
    let partes = |v: &str| -> Vec<u64> { v.split('.').map(|p| p.trim().parse().unwrap_or(0)).collect() };
    let (pa, pb) = (partes(a), partes(b));
    for i in 0..pa.len().max(pb.len()) {
        let ordem = pa.get(i).unwrap_or(&0).cmp(pb.get(i).unwrap_or(&0));
        if ordem != Ordering::Equal {
            return ordem;
        }
    }
    Ordering::Equal
}
