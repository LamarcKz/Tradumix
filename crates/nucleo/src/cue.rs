//! Leitura e escrita de `.cue` (imagem de CD em `.bin`, um arquivo por faixa no padrão Redump).

use std::fs;
use std::path::{Path, PathBuf};

use crate::Erro;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoFaixa {
    Mode1_2352,
    Mode2_2352,
    Audio,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Faixa {
    pub numero: u32,
    pub tipo: TipoFaixa,
    /// Caminho do `.bin`, resolvido a partir da pasta do `.cue`.
    pub arquivo: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Cue {
    pub caminho: PathBuf,
    pub faixas: Vec<Faixa>,
    /// O texto original, para `escrever_cue` manter tudo menos os nomes dos arquivos.
    texto: String,
}

impl Cue {
    /// A primeira faixa que não é de áudio.
    pub fn faixa_de_dados(&self) -> Option<&Faixa> {
        self.faixas.iter().find(|f| f.tipo != TipoFaixa::Audio)
    }

    /// Os `.bin` citados no `.cue`, na ordem, sem repetir.
    pub fn arquivos(&self) -> Vec<PathBuf> {
        let mut lista: Vec<PathBuf> = Vec::new();
        for f in &self.faixas {
            if !lista.contains(&f.arquivo) {
                lista.push(f.arquivo.clone());
            }
        }
        lista
    }
}

pub fn ler_cue(caminho: &Path) -> Result<Cue, Erro> {
    let invalido = |detalhe: String| Erro::CueInvalido(detalhe);
    let bytes = fs::read(caminho).map_err(|e| invalido(e.to_string()))?;
    // UTF-8 e, se não for, Latin-1 (cada byte é o caractere de mesmo código).
    let texto = match String::from_utf8(bytes) {
        Ok(t) => t,
        Err(e) => e.into_bytes().iter().map(|&b| b as char).collect(),
    };
    let texto = texto.strip_prefix('\u{feff}').map(str::to_string).unwrap_or(texto);
    let pasta = caminho.parent().unwrap_or(Path::new(""));
    let mut faixas = Vec::new();
    let mut arquivo_atual: Option<PathBuf> = None;
    for linha in texto.lines() {
        let linha = linha.trim();
        let (comando, resto) = linha.split_once(char::is_whitespace).unwrap_or((linha, ""));
        match comando.to_ascii_uppercase().as_str() {
            "FILE" => {
                let nome = nome_do_file(resto.trim()).ok_or_else(|| invalido(format!("linha FILE sem nome: {linha}")))?;
                arquivo_atual = Some(pasta.join(nome));
            }
            "TRACK" => {
                let mut partes = resto.split_whitespace();
                let numero = partes
                    .next()
                    .and_then(|n| n.parse().ok())
                    .ok_or_else(|| invalido(format!("linha TRACK sem número: {linha}")))?;
                let tipo = match partes.next().map(str::to_ascii_uppercase).as_deref() {
                    Some("MODE1/2352") => TipoFaixa::Mode1_2352,
                    Some("MODE2/2352") => TipoFaixa::Mode2_2352,
                    Some("AUDIO") => TipoFaixa::Audio,
                    _ => return Err(invalido(format!("tipo de faixa não suportado: {linha}"))),
                };
                let arquivo = arquivo_atual.clone().ok_or_else(|| invalido(format!("TRACK antes de FILE: {linha}")))?;
                faixas.push(Faixa { numero, tipo, arquivo });
            }
            _ => {}
        }
    }
    if faixas.is_empty() {
        return Err(invalido("nenhuma faixa".into()));
    }
    Ok(Cue { caminho: caminho.to_path_buf(), faixas, texto })
}

/// O nome depois de `FILE`: entre aspas, ou tudo até o último espaço (o tipo, `BINARY`).
fn nome_do_file(resto: &str) -> Option<String> {
    let nome = if let Some(sem_aspa) = resto.strip_prefix('"') {
        &sem_aspa[..sem_aspa.find('"')?]
    } else {
        resto.rsplit_once(char::is_whitespace).map(|(n, _)| n.trim()).unwrap_or(resto)
    };
    (!nome.is_empty()).then(|| nome.to_string())
}

/// O mesmo `.cue`, com o nome de cada `FILE` trocado pelo de `nomes`, na ordem.
pub fn escrever_cue(cue: &Cue, nomes: &[String]) -> String {
    let mut nomes = nomes.iter();
    let mut saida = String::with_capacity(cue.texto.len());
    for linha in cue.texto.split_inclusive('\n') {
        let conteudo = linha.trim_end_matches(['\r', '\n']);
        let fim = &linha[conteudo.len()..];
        let sem_recuo = conteudo.trim_start();
        let recuo = &conteudo[..conteudo.len() - sem_recuo.len()];
        let (comando, resto) = sem_recuo.split_once(char::is_whitespace).unwrap_or((sem_recuo, ""));
        if comando.eq_ignore_ascii_case("FILE") {
            if let Some(novo) = nomes.next() {
                let resto = resto.trim();
                let tipo = match resto.strip_prefix('"') {
                    Some(s) => s.find('"').map(|i| s[i + 1..].trim()).unwrap_or(""),
                    None => resto.rsplit_once(char::is_whitespace).map(|(_, t)| t).unwrap_or(""),
                };
                saida.push_str(&format!("{recuo}FILE \"{novo}\" {tipo}{fim}"));
                continue;
            }
        }
        saida.push_str(linha);
    }
    saida
}
