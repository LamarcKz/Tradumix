//! Aviso de versão nova do app: procura no GitHub (o `latest.json` da última release, assinado com a
//! chave do Tradumix) e, se a pessoa quiser, baixa e instala. Na versão portátil não instala: abre a
//! página de download. No `npm run dev` nunca procura.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::pasta_dados;

/// Onde baixar a versão portátil.
const PAGINA_DE_DOWNLOAD: &str = "https://github.com/LamarcKz/Tradumix/releases/latest";

/// A versão encontrada na última procura, guardada para instalar.
#[derive(Default)]
pub struct Atualizacao(Mutex<Option<Update>>);

#[derive(Serialize)]
pub struct VersaoNova {
    pub versao: String,
    pub notas: String,
    pub portatil: bool,
}

fn e_portatil() -> bool {
    std::env::current_exe().is_ok_and(|exe| pasta_dados::e_portatil(&exe, cfg!(debug_assertions)))
}

/// Procura uma versão nova. Sem versão nova, sem internet, com assinatura errada ou no `npm run dev`:
/// `None` (o aviso simplesmente não aparece).
#[tauri::command]
pub async fn procurar_versao_nova(app: AppHandle, estado: State<'_, Atualizacao>) -> Result<Option<VersaoNova>, String> {
    if cfg!(debug_assertions) {
        return Ok(None);
    }
    let Ok(atualizador) = app.updater() else { return Ok(None) };
    let Ok(Some(nova)) = atualizador.check().await else { return Ok(None) };
    let resposta = VersaoNova { versao: nova.version.clone(), notas: nova.body.clone().unwrap_or_default(), portatil: e_portatil() };
    *estado.0.lock().unwrap_or_else(|e| e.into_inner()) = Some(nova);
    Ok(Some(resposta))
}

/// Baixa e instala a versão encontrada e reinicia o app. Na portátil, abre a página de download.
#[tauri::command]
pub async fn instalar_versao_nova(app: AppHandle, estado: State<'_, Atualizacao>) -> Result<(), String> {
    if e_portatil() {
        return app.opener().open_url(PAGINA_DE_DOWNLOAD, None::<&str>).map_err(|e| e.to_string());
    }
    let nova = estado.0.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let nova = nova.ok_or("Nenhuma versão nova encontrada.")?;
    nova.download_and_install(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
    app.restart()
}
