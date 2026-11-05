//! Acesso à internet. Fica atrás do trait `Rede` para os testes usarem um servidor local.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::{ArquivoRemoto, Erro};

pub trait Rede: Send + Sync {
    /// Baixa `url` para `destino`, sem passar de `limite` bytes. Devolve quantos bytes vieram.
    fn baixar(&self, url: &str, destino: &mut dyn Write, limite: u64, progresso: &mut dyn FnMut(u64)) -> Result<u64, Erro>;
}

pub struct RedeUreq {
    agente: ureq::Agent,
}

impl RedeUreq {
    /// 30 s sem chegar nenhum byte (ou sem resposta do servidor) é erro. Não há limite de tempo
    /// total: um patch grande numa conexão lenta pode levar o tempo que precisar.
    pub fn novo() -> RedeUreq {
        RedeUreq::com_tempos(Duration::from_secs(30), Duration::from_secs(30))
    }

    pub fn com_tempos(por_leitura: Duration, resposta: Duration) -> RedeUreq {
        let config = ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_recv_response(Some(resposta))
            .timeout_per_read(Some(por_leitura))
            .build();
        RedeUreq { agente: config.into() }
    }
}

impl Default for RedeUreq {
    fn default() -> Self {
        RedeUreq::novo()
    }
}

impl Rede for RedeUreq {
    fn baixar(&self, url: &str, destino: &mut dyn Write, limite: u64, progresso: &mut dyn FnMut(u64)) -> Result<u64, Erro> {
        let mut resposta = self.agente.get(url).call().map_err(|e| Erro::Rede(e.to_string()))?;
        let mut leitor = resposta.body_mut().as_reader();
        let mut buffer = vec![0u8; 64 * 1024];
        let mut total = 0u64;
        loop {
            let n = leitor.read(&mut buffer).map_err(|e| Erro::Rede(e.to_string()))?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > limite {
                return Err(Erro::DownloadInvalido("o arquivo é maior que o esperado".into()));
            }
            destino.write_all(&buffer[..n]).map_err(|e| Erro::Io(e.to_string()))?;
            progresso(total);
        }
        Ok(total)
    }
}

pub(crate) fn sha256_arquivo(caminho: &Path) -> Option<String> {
    let mut arquivo = File::open(caminho).ok()?;
    let mut sha = Sha256::new();
    io::copy(&mut arquivo, &mut sha).ok()?;
    Some(hex(&sha.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Grava no arquivo e calcula o SHA-256 ao mesmo tempo; para quando `cancelar` liga.
struct Gravador<'a> {
    arquivo: File,
    sha: Sha256,
    cancelar: Option<&'a AtomicBool>,
}

impl Write for Gravador<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Não usar ErrorKind::Interrupted: o write_all repete a gravação nesse caso.
        if self.cancelar.is_some_and(|c| c.load(Ordering::Relaxed)) {
            return Err(io::Error::other("cancelado"));
        }
        self.sha.update(buf);
        self.arquivo.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.arquivo.flush()
    }
}

/// Baixa para `<destino>.parcial`, confere tamanho e SHA-256 e só então renomeia para `destino`.
/// Em qualquer falha o parcial é apagado e `destino` não é tocado.
pub(crate) fn baixar_conferido(
    rede: &dyn Rede,
    remoto: &ArquivoRemoto,
    destino: &Path,
    limite: u64,
    progresso: &mut dyn FnMut(u64),
    cancelar: Option<&AtomicBool>,
) -> Result<(), Erro> {
    if remoto.tamanho > limite {
        return Err(Erro::DownloadInvalido(format!("o arquivo declarado tem {} bytes, acima do limite", remoto.tamanho)));
    }
    let mut parcial = destino.as_os_str().to_owned();
    parcial.push(".parcial");
    let parcial = std::path::PathBuf::from(parcial);
    let resultado = (|| {
        let mut gravador = Gravador { arquivo: File::create(&parcial)?, sha: Sha256::new(), cancelar };
        let total = rede.baixar(&remoto.url, &mut gravador, remoto.tamanho, progresso)?;
        gravador.flush()?;
        let sha = hex(&gravador.sha.finalize());
        drop(gravador.arquivo);
        if total != remoto.tamanho {
            return Err(Erro::DownloadInvalido(format!("vieram {total} bytes, eram esperados {}", remoto.tamanho)));
        }
        if !sha.eq_ignore_ascii_case(&remoto.sha256) {
            return Err(Erro::DownloadInvalido("o arquivo baixado não confere com o SHA-256 do catálogo".into()));
        }
        fs::rename(&parcial, destino)?;
        Ok(())
    })();
    if resultado.is_err() {
        let _ = fs::remove_file(&parcial);
        if cancelar.is_some_and(|c| c.load(Ordering::Relaxed)) {
            return Err(Erro::Cancelado);
        }
    }
    resultado
}
