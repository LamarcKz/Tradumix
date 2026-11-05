//! Leitura de arquivos de dentro de uma ISO9660, sem montar a imagem.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::Erro;

const SETOR: u64 = 2048;

/// Uma ISO aberta só para leitura.
pub struct Iso {
    arquivo: File,
    tamanho: u64,
    raiz: (u32, u32),
}

impl Iso {
    pub fn abrir(caminho: &Path) -> Result<Iso, Erro> {
        let arquivo = File::open(caminho)?;
        let tamanho = arquivo.metadata()?.len();
        let mut iso = Iso { arquivo, tamanho, raiz: (0, 0) };
        iso.raiz = iso.achar_raiz()?;
        Ok(iso)
    }

    pub fn tamanho(&self) -> u64 {
        self.tamanho
    }

    /// Lê um arquivo pelo caminho dentro da ISO (`SYSTEM.CNF`, `PASTA/ARQ.BIN`).
    /// Não diferencia maiúsculas e aceita ou não o sufixo `;1`.
    pub fn ler_arquivo(&mut self, caminho: &str) -> Result<Vec<u8>, Erro> {
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
                let mut dados = vec![0u8; tamanho as usize];
                self.ler(setor as u64 * SETOR, &mut dados)?;
                return Ok(dados);
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
            if (n + 1) * SETOR > self.tamanho {
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

    fn ler(&mut self, pos: u64, dest: &mut [u8]) -> Result<(), Erro> {
        if pos + dest.len() as u64 > self.tamanho {
            return Err(Erro::IsoInvalida);
        }
        self.arquivo.seek(SeekFrom::Start(pos))?;
        self.arquivo.read_exact(dest)?;
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
