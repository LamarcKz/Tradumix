// Sem janela de console no app de release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod comandos;
mod config;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dados = app.path().app_data_dir()?;
            app.manage(comandos::Estado::novo(dados));
            Ok(())
        })
        .on_window_event(|janela, evento| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = evento {
                if janela.state::<comandos::Estado>().pedir_fechamento() {
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            comandos::listar_jogos,
            comandos::sincronizar_catalogo,
            comandos::escolher_pasta_rascunhos,
            comandos::definir_mostrar_trailers,
            comandos::conferir_iso,
            comandos::traduzir,
            comandos::cancelar_traducao,
            comandos::arquivo_existe,
            comandos::abrir_pasta,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Tradumix");
}
