//! Aplicar a tradução: conferir a ISO, rodar o xdelta3 e verificar o resultado.

use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::{Erro, Instalacao, Pacote, Rede};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const MARGEM_ESPACO: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Etapa {
    Conferindo,
    Baixando,
    Aplicando,
    Verificando,
}

#[derive(Debug, Clone)]
pub struct Pedido {
    pub iso_original: PathBuf,
    pub saida: PathBuf,
    pub sobrescrever: bool,
    pub xdelta3: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resultado {
    pub saida: PathBuf,
    /// Hash do RetroAchievements da cópia gerada (`None` em plataforma sem RetroAchievements).
    pub hash_ra: Option<String>,
    /// O hash continua o da original (`None` em plataforma sem RetroAchievements).
    pub compativel_ra: Option<bool>,
    pub segundos: f64,
}

/// Nome sugerido para a cópia traduzida, na mesma pasta da original (a plataforma decide).
pub fn nome_saida(pacote: &Pacote, original: &Path) -> PathBuf {
    pacote.alvo.nome_saida(&pacote.ficha, original)
}

/// Confere se o arquivo escolhido é a cópia que a tradução espera, sem lê-lo inteiro.
pub fn conferir(pacote: &Pacote, original: &Path) -> Result<(), Erro> {
    pacote.alvo.conferir(&pacote.ficha, original)
}

/// Aplica sem internet: o patch precisa já estar na pasta do pacote.
pub fn aplicar(
    pacote: &Pacote,
    pedido: &Pedido,
    progresso: &mut dyn FnMut(Etapa, f32),
    cancelar: &AtomicBool,
) -> Result<Resultado, Erro> {
    aplicar_com_rede(pacote, pedido, None, progresso, cancelar)
}

/// Como `aplicar`, mas baixa o patch antes (etapa Baixando) quando ele ainda não está na pasta.
pub fn aplicar_com_rede(
    pacote: &Pacote,
    pedido: &Pedido,
    rede: Option<&dyn Rede>,
    progresso: &mut dyn FnMut(Etapa, f32),
    cancelar: &AtomicBool,
) -> Result<Resultado, Erro> {
    match &pacote.ficha.instalacao {
        Instalacao::ImagemXdelta { .. } => instalar_imagem_xdelta(pacote, pedido, rede, progresso, cancelar),
    }
}

/// `imagem-xdelta`: confere a original, baixa o patch se precisar, gera a cópia traduzida ao lado
/// (num `.parcial` renomeado no fim) e verifica o resultado pela plataforma.
fn instalar_imagem_xdelta(
    pacote: &Pacote,
    pedido: &Pedido,
    rede: Option<&dyn Rede>,
    progresso: &mut dyn FnMut(Etapa, f32),
    cancelar: &AtomicBool,
) -> Result<Resultado, Erro> {
    let inicio = Instant::now();
    progresso(Etapa::Conferindo, 0.0);
    if mesmo_arquivo(&pedido.saida, &pedido.iso_original) {
        return Err(Erro::SaidaIgualOriginal);
    }
    if pedido.saida.exists() && !pedido.sobrescrever {
        return Err(Erro::SaidaJaExiste(pedido.saida.clone()));
    }
    conferir(pacote, &pedido.iso_original)?;
    let esperado = pacote.alvo.tamanho_resultado();
    let pasta_saida = pedido.saida.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let livre = espaco_livre(pasta_saida)?;
    if livre < esperado + MARGEM_ESPACO {
        return Err(Erro::EspacoInsuficiente { precisa: esperado + MARGEM_ESPACO, livre });
    }
    progresso(Etapa::Conferindo, 1.0);

    if !pacote.patch_disponivel() {
        let rede = rede.ok_or(Erro::PrecisaInternet)?;
        progresso(Etapa::Baixando, 0.0);
        crate::baixar_patch(rede, pacote, &mut |f| progresso(Etapa::Baixando, f), cancelar)?;
    }

    let parcial = caminho_parcial(&pedido.saida);
    let resultado = gerar(pacote, pedido, &parcial, esperado, progresso, cancelar).and_then(|()| {
        progresso(Etapa::Verificando, 0.0);
        let hash_ra = pacote.alvo.verificar_resultado(&parcial)?;
        fs::rename(&parcial, &pedido.saida)?;
        progresso(Etapa::Verificando, 1.0);
        Ok(Resultado {
            saida: pedido.saida.clone(),
            compativel_ra: hash_ra.as_deref().zip(pacote.alvo.hash_ra_esperado()).map(|(a, b)| a.eq_ignore_ascii_case(b)),
            hash_ra,
            segundos: inicio.elapsed().as_secs_f64(),
        })
    });
    if resultado.is_err() {
        let _ = fs::remove_file(&parcial);
    }
    resultado
}

/// Roda o xdelta3 gravando em `parcial` e informa o progresso pelo tamanho do arquivo.
fn gerar(
    pacote: &Pacote,
    pedido: &Pedido,
    parcial: &Path,
    esperado: u64,
    progresso: &mut dyn FnMut(Etapa, f32),
    cancelar: &AtomicBool,
) -> Result<(), Erro> {
    progresso(Etapa::Aplicando, 0.0);
    if cancelar.load(Ordering::Relaxed) {
        return Err(Erro::Cancelado);
    }
    let mut filho = Command::new(&pedido.xdelta3)
        .args(["-d", "-f", "-s"])
        .arg(&pedido.iso_original)
        .arg(pacote.patch())
        .arg(parcial)
        .env_remove("XDELTA")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| Erro::XdeltaFalhou(format!("não foi possível iniciar o xdelta3: {e}")))?;
    // Se o app fechar de repente, o Windows encerra o xdelta3 junto (o grupo vive até o fim desta função).
    let _grupo = crate::GrupoDeProcessos::novo().and_then(|g| g.incluir(&filho).map(|()| g)).ok();
    let mut stderr = filho.stderr.take().expect("stderr do xdelta3");
    let leitor = std::thread::spawn(move || {
        let mut texto = String::new();
        let _ = stderr.read_to_string(&mut texto);
        texto
    });

    let status = loop {
        if cancelar.load(Ordering::Relaxed) {
            let _ = filho.kill();
            let _ = filho.wait();
            let _ = leitor.join();
            return Err(Erro::Cancelado);
        }
        if let Some(status) = filho.try_wait()? {
            break status;
        }
        let atual = fs::metadata(parcial).map(|m| m.len()).unwrap_or(0);
        progresso(Etapa::Aplicando, fracao(atual, esperado));
        std::thread::sleep(Duration::from_millis(200));
    };
    let mensagem = leitor.join().unwrap_or_default();
    if !status.success() {
        // O xdelta3 3.2.x avisa quando a "original" já é o resultado do patch.
        if mensagem.contains("already matches the target of this patch") {
            return Err(Erro::JaTraduzida);
        }
        let detalhe = mensagem.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("erro desconhecido").trim().to_string();
        return Err(Erro::XdeltaFalhou(detalhe));
    }
    progresso(Etapa::Aplicando, 1.0);
    Ok(())
}

fn fracao(atual: u64, esperado: u64) -> f32 {
    if esperado == 0 { 0.0 } else { (atual as f64 / esperado as f64).clamp(0.0, 1.0) as f32 }
}

fn caminho_parcial(saida: &Path) -> PathBuf {
    let mut nome = OsString::from(saida.as_os_str());
    nome.push(".parcial");
    PathBuf::from(nome)
}

fn mesmo_arquivo(a: &Path, b: &Path) -> bool {
    let canonico = |p: &Path| -> Option<PathBuf> {
        if let Ok(c) = p.canonicalize() {
            return Some(c);
        }
        let pai = p.parent()?.canonicalize().ok()?;
        Some(pai.join(p.file_name()?))
    };
    match (canonico(a), canonico(b)) {
        (Some(x), Some(y)) => x.to_string_lossy().eq_ignore_ascii_case(&y.to_string_lossy()),
        _ => false,
    }
}

fn espaco_livre(pasta: &Path) -> Result<u64, Erro> {
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    use windows::core::HSTRING;
    let caminho = HSTRING::from(pasta.as_os_str());
    let mut livre = 0u64;
    unsafe { GetDiskFreeSpaceExW(&caminho, Some(&mut livre), None, None) }
        .map_err(|e| Erro::Io(format!("não foi possível consultar o espaço livre: {e}")))?;
    Ok(livre)
}
