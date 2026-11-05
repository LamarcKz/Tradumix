//! Baixa o patch de uma tradução do catálogo na hora de traduzir.

use std::sync::atomic::AtomicBool;

use crate::rede::baixar_conferido;
use crate::{Erro, Pacote, Rede};

const LIMITE_PATCH: u64 = 2 * 1024 * 1024 * 1024;

/// Já disponível: não faz nada. Senão baixa, confere tamanho e SHA-256 e guarda na pasta do pacote.
pub fn baixar_patch(rede: &dyn Rede, pacote: &Pacote, progresso: &mut dyn FnMut(f32), cancelar: &AtomicBool) -> Result<(), Erro> {
    if pacote.patch_disponivel() {
        return Ok(());
    }
    let download = pacote.ficha.download.as_ref().ok_or(Erro::PrecisaInternet)?;
    let total = download.patch.tamanho.max(1) as f64;
    progresso(0.0);
    baixar_conferido(
        rede,
        &download.patch,
        &pacote.patch(),
        LIMITE_PATCH,
        &mut |vieram| progresso((vieram as f64 / total).min(1.0) as f32),
        Some(cancelar),
    )?;
    progresso(1.0);
    Ok(())
}
