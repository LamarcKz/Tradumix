//! Leitura de arquivos de dentro de uma ISO9660, sem montar a imagem.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::Erro;
use crate::cue::TipoFaixa;

const SETOR: u64 = 2048;

/// Como os setores lógicos de 2048 bytes estão gravados no arquivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setores {
    /// ISO comum: cada setor tem só os 2048 bytes de dados.
    Iso2048,
    /// Imagem de CD em `.bin`: setores de `tamanho` bytes com os dados a partir de `dados`.
    Bin { tamanho: u64, dados: u64 },
}

impl Setores {
    pub fn do_tipo(tipo: TipoFaixa) -> Setores {
        match tipo {
            TipoFaixa::Mode1_2352 => Setores::Bin { tamanho: 2352, dados: 16 },
            // Faixa de áudio não tem sistema de arquivos; a leitura falha como ISO inválida.
            TipoFaixa::Mode2_2352 | TipoFaixa::Audio => Setores::Bin { tamanho: 2352, dados: 24 },
        }
    }
}

/// Uma ISO aberta só para leitura.
pub struct Iso {
    arquivo: File,
    tamanho: u64,
    setores: Setores,
    raiz: (u32, u32),
}

impl Iso {
    pub fn abrir(caminho: &Path) -> Result<Iso, Erro> {
        Iso::abrir_com(caminho, Setores::Iso2048)
    }

    pub fn abrir_com(caminho: &Path, setores: Setores) -> Result<Iso, Erro> {
        let arquivo = File::open(caminho)?;
        let tamanho = arquivo.metadata()?.len();
        let mut iso = Iso { arquivo, tamanho, setores, raiz: (0, 0) };
        iso.raiz = iso.achar_raiz()?;
        Ok(iso)
    }

    /// Tamanho do arquivo no disco.
    pub fn tamanho(&self) -> u64 {
        self.tamanho
    }

    /// Bytes de dados lógicos (setores de 2048) que cabem no arquivo.
    fn tamanho_logico(&self) -> u64 {
        match self.setores {
            Setores::Iso2048 => self.tamanho,
            Setores::Bin { tamanho, .. } => self.tamanho / tamanho * SETOR,
        }
    }

    /// Lê um arquivo pelo caminho dentro da ISO (`SYSTEM.CNF`, `PASTA/ARQ.BIN`).
    /// Não diferencia maiúsculas e aceita ou não o sufixo `;1`.
    pub fn ler_arquivo(&mut self, caminho: &str) -> Result<Vec<u8>, Erro> {
        self.ler_a_partir_de(caminho, 0)
    }

    /// Como `ler_arquivo`, mas lê pelo menos `minimo` bytes a partir do início do arquivo,
    /// seguindo pelos setores de depois dele (como o rcheevos faz com o executável do PS1).
    /// Para no fim da imagem.
    pub(crate) fn ler_a_partir_de(&mut self, caminho: &str, minimo: u64) -> Result<Vec<u8>, Erro> {
        let (setor, tamanho) = self.localizar(caminho)?;
        let inicio = setor as u64 * SETOR;
        let quanto = (tamanho as u64).max(minimo.min(self.tamanho_logico().saturating_sub(inicio)));
        let mut dados = vec![0u8; quanto as usize];
        self.ler(inicio, &mut dados)?;
        Ok(dados)
    }

    /// Setor inicial e tamanho de um arquivo.
    fn localizar(&mut self, caminho: &str) -> Result<(u32, u32), Erro> {
        let partes: Vec<&str> = caminho
            .split(['/', '\\'])
            .filter(|p| !p.is_empty())
            .collect();
        let nao_achou = || Erro::ArquivoNaoEncontrado(caminho.to_string());
        if partes.is_empty() {
            return Err(nao_achou());
        }
        let mut dir = self.raiz;
        for (i, parte) in partes.iter().enumerate() {
            let ultimo = i == partes.len() - 1;
            let (setor, tamanho, pasta) = self.procurar(dir, parte)?.ok_or_else(nao_achou)?;
            if ultimo {
                if pasta {
                    return Err(nao_achou());
                }
                return Ok((setor, tamanho));
            }
            if !pasta {
                return Err(nao_achou());
            }
            dir = (setor, tamanho);
        }
        Err(nao_achou())
    }

    fn achar_raiz(&mut self) -> Result<(u32, u32), Erro> {
        let mut setor = [0u8; SETOR as usize];
        for n in 16..64u64 {
            if (n + 1) * SETOR > self.tamanho_logico() {
                break;
            }
            self.ler(n * SETOR, &mut setor)?;
            if &setor[1..6] != b"CD001" {
                break;
            }
            match setor[0] {
                1 => {
                    let r = &setor[156..190];
                    return Ok((le32(&r[2..]), le32(&r[10..])));
                }
                255 => break,
                _ => {}
            }
        }
        Err(Erro::IsoInvalida)
    }

    /// Procura `nome` no diretório; devolve (setor, tamanho, é_pasta).
    fn procurar(&mut self, (inicio, tamanho): (u32, u32), nome: &str) -> Result<Option<(u32, u32, bool)>, Erro> {
        let alvo = normalizar(nome.as_bytes());
        let mut dados = vec![0u8; tamanho as usize];
        self.ler(inicio as u64 * SETOR, &mut dados)?;
        for bloco in dados.chunks(SETOR as usize) {
            let mut pos = 0;
            while pos < bloco.len() {
                let len = bloco[pos] as usize;
                if len == 0 || pos + len > bloco.len() || len < 34 {
                    break;
                }
                let r = &bloco[pos..pos + len];
                let nome_len = r[32] as usize;
                if 33 + nome_len <= len {
                    let nome_reg = &r[33..33 + nome_len];
                    if nome_reg != [0] && nome_reg != [1] && normalizar(nome_reg) == alvo {
                        return Ok(Some((le32(&r[2..]), le32(&r[10..]), r[25] & 2 != 0)));
                    }
                }
                pos += len;
            }
        }
        Ok(None)
    }

    /// Lê `dest.len()` bytes a partir da posição lógica `pos` (setor × 2048 + deslocamento).
    fn ler(&mut self, pos: u64, dest: &mut [u8]) -> Result<(), Erro> {
        if pos + dest.len() as u64 > self.tamanho_logico() {
            return Err(Erro::IsoInvalida);
        }
        match self.setores {
            Setores::Iso2048 => {
                self.arquivo.seek(SeekFrom::Start(pos))?;
                self.arquivo.read_exact(dest)?;
            }
            Setores::Bin { tamanho, dados } => {
                let mut feito = 0usize;
                while feito < dest.len() {
                    let logico = pos + feito as u64;
                    let (setor, dentro) = (logico / SETOR, logico % SETOR);
                    let pedaco = ((SETOR - dentro) as usize).min(dest.len() - feito);
                    self.arquivo.seek(SeekFrom::Start(setor * tamanho + dados + dentro))?;
                    self.arquivo.read_exact(&mut dest[feito..feito + pedaco])?;
                    feito += pedaco;
                }
            }
        }
        Ok(())
    }
}

fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// Nome sem `;versão`, sem ponto final solto e em maiúsculas.
fn normalizar(nome: &[u8]) -> Vec<u8> {
    let sem_versao = match nome.iter().position(|&c| c == b';') {
        Some(i) => &nome[..i],
        None => nome,
    };
    let sem_ponto = sem_versao.strip_suffix(b".").unwrap_or(sem_versao);
    sem_ponto.to_ascii_uppercase()
}
