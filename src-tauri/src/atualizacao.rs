//! Aviso de versão nova do app: procura no GitHub (o `latest.json` da última release) e já baixa o
//! instalador, porque só o download confere a assinatura da chave do Tradumix: o aviso só aparece com a
//! assinatura conferida. Se a pessoa quiser, instala o que já foi baixado. Na versão portátil não instala: abre a
//! página de download. No `npm run dev` nunca procura.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::pasta_dados;

/// Onde baixar a versão portátil.
const PAGINA_DE_DOWNLOAD: &str = "https://github.com/LamarcKz/Tradumix/releases/latest";

/// A versão encontrada na última procura e o instalador já baixado e conferido, guardados para instalar.
#[derive(Default)]
pub struct Atualizacao(Mutex<Option<(Update, Vec<u8>)>>);

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
    let achada = atualizador.check().await;
    let Some((nova, instalador)) = versao_conferida(achada, async |u: &Update| u.download(|_, _| {}, || {}).await).await else {
        return Ok(None);
    };
    let resposta = VersaoNova { versao: nova.version.clone(), notas: nova.body.clone().unwrap_or_default(), portatil: e_portatil() };
    *estado.0.lock().unwrap_or_else(|e| e.into_inner()) = Some((nova, instalador));
    Ok(Some(resposta))
}

/// Só há aviso quando há versão nova e o download dela passou (é no download que o plugin confere a
/// assinatura). Sem versão nova, com erro ao procurar ou com assinatura errada: `None`.
async fn versao_conferida<U, E1, E2>(
    achada: Result<Option<U>, E1>,
    baixar: impl AsyncFnOnce(&U) -> Result<Vec<u8>, E2>,
) -> Option<(U, Vec<u8>)> {
    let nova = achada.ok().flatten()?;
    let instalador = baixar(&nova).await.ok()?;
    Some((nova, instalador))
}

/// Instala a versão já baixada e conferida e reinicia o app. Na portátil, abre a página de download.
#[tauri::command]
pub async fn instalar_versao_nova(app: AppHandle, estado: State<'_, Atualizacao>) -> Result<(), String> {
    if e_portatil() {
        return app.opener().open_url(PAGINA_DE_DOWNLOAD, None::<&str>).map_err(|e| e.to_string());
    }
    let guardada = estado.0.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let (nova, instalador) = guardada.ok_or("Nenhuma versão nova encontrada.")?;
    nova.install(instalador).map_err(|_| "O instalador da versão nova não abriu. Tente de novo ou baixe pela página do Tradumix.".to_string())?;
    app.restart()
}

#[derive(Serialize)]
pub struct VersaoDoApp {
    pub versao: String,
    pub portatil: bool,
}

/// A versão deste app (a do package.json) e se é a portátil, para a tela Sobre.
#[tauri::command]
pub fn versao_do_app(app: AppHandle) -> VersaoDoApp {
    VersaoDoApp { versao: app.package_info().version.to_string(), portatil: e_portatil() }
}

#[cfg(test)]
mod testes {
    use super::*;
    use tauri::async_runtime::block_on;

    #[test]
    fn sem_versao_nova_nao_avisa() {
        let r = block_on(versao_conferida(Ok::<Option<&str>, ()>(None), async |_: &&str| Ok::<_, ()>(vec![1])));
        assert_eq!(r, None);
    }

    #[test]
    fn erro_ao_procurar_nao_avisa() {
        let r = block_on(versao_conferida(Err::<Option<&str>, _>("sem internet"), async |_: &&str| Ok::<_, ()>(vec![1])));
        assert_eq!(r, None);
    }

    #[test]
    fn assinatura_errada_nao_avisa() {
        let r = block_on(versao_conferida(Ok::<_, ()>(Some("0.2.0")), async |_: &&str| Err::<Vec<u8>, _>("assinatura")));
        assert_eq!(r, None);
    }

    #[test]
    fn assinatura_conferida_avisa_e_guarda_o_instalador() {
        let r = block_on(versao_conferida(Ok::<_, ()>(Some("0.2.0")), async |_: &&str| Ok::<_, ()>(vec![7, 7])));
        assert_eq!(r, Some(("0.2.0", vec![7, 7])));
    }
}
