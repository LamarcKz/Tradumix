//! Histórico das ISOs geradas, para avisar quando há versão nova de uma tradução.

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Erro, Pacote, Resultado, comparar_versoes};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entrada {
    pub pacote_id: String,
    pub versao: String,
    pub saida: PathBuf,
    pub data: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Historico {
    pub entradas: Vec<Entrada>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Atualizacao {
    pub pacote_id: String,
    pub versao_gerada: String,
    pub versao_nova: String,
}

impl Historico {
    /// Arquivo ausente ou inválido vira um histórico vazio.
    pub fn carregar(arquivo: &Path) -> Historico {
        fs::read_to_string(arquivo)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    /// Grava num `.tmp` e renomeia, para não deixar o arquivo pela metade.
    pub fn salvar(&self, arquivo: &Path) -> Result<(), Erro> {
        if let Some(pasta) = arquivo.parent() {
            fs::create_dir_all(pasta)?;
        }
        let tmp = arquivo.with_extension("json.tmp");
        let texto = serde_json::to_string_pretty(self).map_err(|e| Erro::Io(e.to_string()))?;
        fs::write(&tmp, texto)?;
        fs::rename(&tmp, arquivo)?;
        Ok(())
    }

    pub fn registrar(&mut self, pacote: &Pacote, resultado: &Resultado, data: &str) {
        self.entradas.push(Entrada {
            pacote_id: pacote.ficha.id.clone(),
            versao: pacote.ficha.versao.clone(),
            saida: resultado.saida.clone(),
            data: data.to_string(),
        });
    }

    /// Para cada tradução já gerada, se algum pacote tem versão maior que a mais nova gerada.
    pub fn atualizacoes(&self, pacotes: &[Pacote]) -> Vec<Atualizacao> {
        let mut gerados: Vec<(&str, &str)> = Vec::new();
        for e in &self.entradas {
            match gerados.iter_mut().find(|(id, _)| *id == e.pacote_id) {
                Some(g) if comparar_versoes(&e.versao, g.1) == Ordering::Greater => g.1 = &e.versao,
                Some(_) => {}
                None => gerados.push((&e.pacote_id, &e.versao)),
            }
        }
        let mut lista = Vec::new();
        for (id, versao_gerada) in gerados {
            let nova = pacotes
                .iter()
                .filter(|p| p.ficha.id == id)
                .map(|p| p.ficha.versao.as_str())
                .max_by(|a, b| comparar_versoes(a, b));
            if let Some(nova) = nova {
                if comparar_versoes(nova, versao_gerada) == Ordering::Greater {
                    lista.push(Atualizacao {
                        pacote_id: id.to_string(),
                        versao_gerada: versao_gerada.to_string(),
                        versao_nova: nova.to_string(),
                    });
                }
            }
        }
        lista
    }
}
