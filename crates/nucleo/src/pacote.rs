//! Ficha de tradução (`ficha.json`) e pacotes guardados em pastas.

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{IdentidadePs2, Regiao};

pub const FORMATO_FICHA: u32 = 1;

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
    pub console: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Origem {
    pub serial: String,
    pub regiao: Regiao,
    pub hash_ra: String,
    pub tamanho: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultadoEsperado {
    pub hash_ra: String,
    pub tamanho: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tradutor {
    pub nome: String,
    #[serde(default)]
    pub link: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arquivos {
    pub patch: String,
    #[serde(default)]
    pub capa: Option<String>,
    #[serde(default)]
    pub fundo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ficha {
    pub formato: u32,
    pub id: String,
    pub jogo: Jogo,
    pub versao: String,
    pub estado: Estado,
    pub idioma: String,
    pub data: String,
    pub origem: Origem,
    pub resultado: ResultadoEsperado,
    pub tradutores: Vec<Tradutor>,
    #[serde(default)]
    pub permissao: Option<String>,
    #[serde(default)]
    pub descricao: String,
    #[serde(default)]
    pub traduzido: Vec<String>,
    #[serde(default)]
    pub limitacoes: Vec<String>,
    pub arquivos: Arquivos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Imagem {
    Capa,
    Fundo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pacote {
    pub ficha: Ficha,
    pub pasta: PathBuf,
}

impl Pacote {
    pub fn patch(&self) -> PathBuf {
        self.pasta.join(&self.ficha.arquivos.patch)
    }

    pub fn imagem(&self, qual: Imagem) -> Option<PathBuf> {
        let nome = match qual {
            Imagem::Capa => self.ficha.arquivos.capa.as_ref(),
            Imagem::Fundo => self.ficha.arquivos.fundo.as_ref(),
        }?;
        Some(self.pasta.join(nome))
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JogoNoCatalogo {
    pub id: String,
    pub nome: String,
    pub console: String,
    pub pacotes: Vec<String>,
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
    if ficha.formato != FORMATO_FICHA {
        return Err(format!("formato de ficha {} não suportado (esperado {FORMATO_FICHA})", ficha.formato));
    }
    for (campo, valor) in [("id", &ficha.id), ("jogo.id", &ficha.jogo.id), ("jogo.nome", &ficha.jogo.nome), ("versao", &ficha.versao)] {
        if valor.trim().is_empty() {
            return Err(format!("o campo {campo} está vazio"));
        }
    }
    for (campo, hash) in [("origem.hash_ra", &ficha.origem.hash_ra), ("resultado.hash_ra", &ficha.resultado.hash_ra)] {
        if hash.len() != 32 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("{campo} deve ter 32 caracteres hexadecimais"));
        }
    }
    let nomes = [Some(&ficha.arquivos.patch), ficha.arquivos.capa.as_ref(), ficha.arquivos.fundo.as_ref()];
    for nome in nomes.into_iter().flatten() {
        if !nome_simples(nome) {
            return Err(format!("o arquivo {nome} precisa estar na própria pasta do pacote"));
        }
    }
    let pacote = Pacote { ficha, pasta: pasta.to_path_buf() };
    if !pacote.patch().is_file() {
        return Err(format!("o arquivo do patch {} não existe", pacote.ficha.arquivos.patch));
    }
    Ok(pacote)
}

/// Só um nome de arquivo, sem pastas, unidade ou `..`.
fn nome_simples(nome: &str) -> bool {
    !nome.is_empty() && nome != "." && nome != ".." && !nome.contains(['/', '\\', ':'])
}

/// Um item por jogo, com os ids das traduções dele, ordenado pelo nome.
pub fn agrupar_por_jogo(pacotes: &[Pacote]) -> Vec<JogoNoCatalogo> {
    let mut jogos: Vec<JogoNoCatalogo> = Vec::new();
    for p in pacotes {
        match jogos.iter_mut().find(|j| j.id == p.ficha.jogo.id) {
            Some(j) => j.pacotes.push(p.ficha.id.clone()),
            None => jogos.push(JogoNoCatalogo {
                id: p.ficha.jogo.id.clone(),
                nome: p.ficha.jogo.nome.clone(),
                console: p.ficha.jogo.console.clone(),
                pacotes: vec![p.ficha.id.clone()],
            }),
        }
    }
    jogos.sort_by(|a, b| a.nome.to_lowercase().cmp(&b.nome.to_lowercase()));
    jogos
}

/// A tradução foi feita para esta ISO?
pub fn serve_para(ficha: &Ficha, id: &IdentidadePs2) -> bool {
    ficha.origem.hash_ra.eq_ignore_ascii_case(&id.hash_ra)
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
