// Sem janela de console no app de release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod atualizacao;
mod banco;
mod comandos;
mod pasta_dados;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(atualizacao::Atualizacao::default())
        .setup(|app| {
            let instalado = app.path().app_data_dir()?;
            let desenvolvimento = cfg!(debug_assertions);
            let dados = pasta_dados::pasta_de_dados(&std::env::current_exe()?, instalado.clone(), desenvolvimento);
            if desenvolvimento {
                pasta_dados::preparar_copia_de_desenvolvimento(&instalado, &dados)?;
            }
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
            comandos::definir_mostrar_trailers,
            comandos::conferir_iso,
            comandos::traduzir,
            comandos::cancelar_traducao,
            comandos::arquivo_existe,
            comandos::abrir_pasta,
            atualizacao::procurar_versao_nova,
            atualizacao::instalar_versao_nova,
            atualizacao::versao_do_app,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Tradumix");
}

#[cfg(test)]
mod testes {
    /// Os links dos créditos e da tela Sobre abrem no navegador pelo plugin opener, que só abre os
    /// endereços da lista da permissão: sem lista, nenhum clique faz nada.
    #[test]
    fn links_https_podem_abrir_no_navegador() {
        let capacidade: serde_json::Value = serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
        let permissoes = capacidade["permissions"].as_array().unwrap();
        let abrir = permissoes
            .iter()
            .find(|p| p["identifier"] == "opener:allow-open-url")
            .expect("permissão opener:allow-open-url com a lista de endereços");
        let enderecos: Vec<&str> = abrir["allow"].as_array().unwrap().iter().filter_map(|e| e["url"].as_str()).collect();
        assert_eq!(enderecos, vec!["https://*"]);
    }
}
