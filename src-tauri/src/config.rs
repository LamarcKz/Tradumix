//! Configurações do app, gravadas em `config.json` na pasta de dados.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Pasta com os pacotes de rascunho (modo de teste).
    #[serde(default)]
    pub pasta_rascunhos: Option<PathBuf>,
}

impl Config {
    pub fn carregar(arquivo: &Path) -> Config {
        fs::read_to_string(arquivo).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    pub fn salvar(&self, arquivo: &Path) -> Result<(), String> {
        if let Some(pasta) = arquivo.parent() {
            fs::create_dir_all(pasta).map_err(|e| e.to_string())?;
        }
        let texto = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(arquivo, texto).map_err(|e| e.to_string())
    }
}
