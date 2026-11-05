//! Os dados da pessoa num banco SQLite (`tradumix.db`, na pasta de dados), como no PlayTrove:
//! preferências, histórico das ISOs geradas e traduções já vistas. O catálogo continua vindo do
//! GitHub, em arquivos na pasta `catalogo`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use nucleo::{Entrada, Historico};
use rusqlite::{Connection, OptionalExtension, params};

/// O arquivo do banco, dentro da pasta de dados.
pub const ARQUIVO_BANCO: &str = "tradumix.db";

/// Versão das tabelas (PRAGMA user_version). Cada mudança de tabela sobe este número.
const VERSAO_DAS_TABELAS: i32 = 1;

pub struct Banco {
    conexao: Mutex<Connection>,
}

impl Banco {
    /// Abre (ou cria) o banco na pasta de dados. Na primeira vez traz o `config.json` e o
    /// `historico.json` das versões antigas. Se o arquivo não for um banco válido, ele é guardado ao
    /// lado como `tradumix.db.corrompido` e começa um banco novo.
    pub fn abrir(pasta: &Path) -> Result<Banco, String> {
        fs::create_dir_all(pasta).map_err(|e| e.to_string())?;
        let arquivo = pasta.join(ARQUIVO_BANCO);
        let conexao = match preparar(&arquivo, pasta) {
            Ok(c) => c,
            Err(_) if arquivo.exists() => {
                fs::rename(&arquivo, pasta.join(format!("{ARQUIVO_BANCO}.corrompido"))).map_err(|e| e.to_string())?;
                preparar(&arquivo, pasta).map_err(|e| e.to_string())?
            }
            Err(e) => return Err(e.to_string()),
        };
        Ok(Banco { conexao: Mutex::new(conexao) })
    }

    /// Um banco só na memória, para quando não dá para gravar na pasta de dados: o app funciona, só
    /// não guarda nada entre uma abertura e outra.
    pub fn na_memoria() -> Banco {
        let conexao = Connection::open_in_memory().expect("banco na memória");
        criar_tabelas(&conexao).expect("tabelas na memória");
        Banco { conexao: Mutex::new(conexao) }
    }

    fn conexao(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conexao.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn preferencia(&self, chave: &str) -> Option<String> {
        self.conexao()
            .query_row("SELECT valor FROM preferencias WHERE chave = ?1", [chave], |l| l.get(0))
            .optional()
            .ok()
            .flatten()
    }

    pub fn definir_preferencia(&self, chave: &str, valor: &str) -> Result<(), String> {
        self.conexao()
            .execute(
                "INSERT INTO preferencias (chave, valor) VALUES (?1, ?2) ON CONFLICT(chave) DO UPDATE SET valor = excluded.valor",
                params![chave, valor],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// O histórico das ISOs geradas, na ordem em que foram geradas.
    pub fn historico(&self) -> Historico {
        let conexao = self.conexao();
        let Ok(mut consulta) = conexao.prepare("SELECT pacote_id, versao, saida, data FROM historico ORDER BY id") else {
            return Historico::default();
        };
        let entradas = consulta
            .query_map([], |l| {
                Ok(Entrada { pacote_id: l.get(0)?, versao: l.get(1)?, saida: PathBuf::from(l.get::<_, String>(2)?), data: l.get(3)? })
            })
            .map(|linhas| linhas.filter_map(Result::ok).collect())
            .unwrap_or_default();
        Historico { entradas }
    }

    pub fn registrar_traducao(&self, entrada: &Entrada) -> Result<(), String> {
        inserir_historico(&self.conexao(), entrada).map_err(|e| e.to_string())
    }

    /// As traduções que a pessoa já viu na lista.
    pub fn vistos(&self) -> HashSet<String> {
        let conexao = self.conexao();
        let Ok(mut consulta) = conexao.prepare("SELECT traducao_id FROM vistos") else { return HashSet::new() };
        consulta
            .query_map([], |l| l.get::<_, String>(0))
            .map(|linhas| linhas.filter_map(Result::ok).collect())
            .unwrap_or_default()
    }

    pub fn marcar_vistos(&self, ids: &[String]) -> Result<(), String> {
        let mut conexao = self.conexao();
        let transacao = conexao.transaction().map_err(|e| e.to_string())?;
        for id in ids {
            transacao.execute("INSERT OR IGNORE INTO vistos (traducao_id) VALUES (?1)", [id]).map_err(|e| e.to_string())?;
        }
        transacao.commit().map_err(|e| e.to_string())
    }
}

/// Abre o arquivo, cria as tabelas que faltam e, num banco novo, traz os arquivos antigos.
fn preparar(arquivo: &Path, pasta: &Path) -> rusqlite::Result<Connection> {
    let conexao = Connection::open(arquivo)?;
    let versao: i32 = conexao.query_row("PRAGMA user_version", [], |l| l.get(0))?;
    if versao < VERSAO_DAS_TABELAS {
        // Tudo numa transação: se parar no meio, na próxima abertura começa de novo sem duplicar.
        let transacao = conexao.unchecked_transaction()?;
        criar_tabelas(&transacao)?;
        if versao == 0 {
            trazer_arquivos_antigos(&transacao, pasta)?;
        }
        transacao.pragma_update(None, "user_version", VERSAO_DAS_TABELAS)?;
        transacao.commit()?;
    }
    Ok(conexao)
}

fn criar_tabelas(conexao: &Connection) -> rusqlite::Result<()> {
    conexao.execute_batch(
        "CREATE TABLE IF NOT EXISTS preferencias (chave TEXT PRIMARY KEY, valor TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS historico (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             pacote_id TEXT NOT NULL,
             versao TEXT NOT NULL,
             saida TEXT NOT NULL,
             data TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS vistos (traducao_id TEXT PRIMARY KEY);",
    )
}

fn inserir_historico(conexao: &Connection, e: &Entrada) -> rusqlite::Result<()> {
    conexao
        .execute(
            "INSERT INTO historico (pacote_id, versao, saida, data) VALUES (?1, ?2, ?3, ?4)",
            params![e.pacote_id, e.versao, e.saida.to_string_lossy(), e.data],
        )
        .map(|_| ())
}

/// O `config.json` e o `historico.json` das versões antigas (até a 0.1.0). Arquivo ausente ou
/// inválido é ignorado; os arquivos ficam onde estão.
fn trazer_arquivos_antigos(conexao: &Connection, pasta: &Path) -> rusqlite::Result<()> {
    let ler = |nome: &str| fs::read_to_string(pasta.join(nome)).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok());
    if let Some(config) = ler("config.json") {
        if let Some(mostrar) = config.get("mostrar_trailers").and_then(|v| v.as_bool()) {
            conexao.execute("INSERT OR REPLACE INTO preferencias VALUES ('mostrar_trailers', ?1)", [if mostrar { "sim" } else { "nao" }])?;
        }
        if let Some(url) = config.get("url_catalogo").and_then(|v| v.as_str()) {
            conexao.execute("INSERT OR REPLACE INTO preferencias VALUES ('url_catalogo', ?1)", [url])?;
        }
    }
    if let Some(historico) = ler("historico.json").and_then(|v| serde_json::from_value::<Historico>(v).ok()) {
        for e in &historico.entradas {
            inserir_historico(conexao, e)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn entrada(id: &str, versao: &str) -> Entrada {
        Entrada { pacote_id: id.into(), versao: versao.into(), saida: PathBuf::from(format!("C:/jogos/{id}.iso")), data: "2026-11-05".into() }
    }

    #[test]
    fn preferencias_ficam_salvas() {
        let pasta = tempfile::tempdir().unwrap();
        let banco = Banco::abrir(pasta.path()).unwrap();
        assert_eq!(banco.preferencia("mostrar_trailers"), None);
        banco.definir_preferencia("mostrar_trailers", "nao").unwrap();
        banco.definir_preferencia("mostrar_trailers", "sim").unwrap();
        drop(banco);
        assert_eq!(Banco::abrir(pasta.path()).unwrap().preferencia("mostrar_trailers").as_deref(), Some("sim"));
        assert!(pasta.path().join(ARQUIVO_BANCO).is_file());
    }

    #[test]
    fn historico_fica_salvo() {
        let pasta = tempfile::tempdir().unwrap();
        let banco = Banco::abrir(pasta.path()).unwrap();
        banco.registrar_traducao(&entrada("jak2-usa-ptbr", "1.0")).unwrap();
        banco.registrar_traducao(&entrada("jak2-usa-ptbr", "1.1")).unwrap();
        drop(banco);
        let historico = Banco::abrir(pasta.path()).unwrap().historico();
        assert_eq!(historico.entradas, vec![entrada("jak2-usa-ptbr", "1.0"), entrada("jak2-usa-ptbr", "1.1")]);
    }

    #[test]
    fn vistos_ficam_salvos_sem_repetir() {
        let pasta = tempfile::tempdir().unwrap();
        let banco = Banco::abrir(pasta.path()).unwrap();
        banco.marcar_vistos(&["jak2-usa-ptbr".to_string(), "sly-ptbr".to_string()]).unwrap();
        banco.marcar_vistos(&["jak2-usa-ptbr".to_string()]).unwrap();
        drop(banco);
        let vistos = Banco::abrir(pasta.path()).unwrap().vistos();
        assert_eq!(vistos.len(), 2);
        assert!(vistos.contains("sly-ptbr"));
    }

    #[test]
    fn traz_o_config_e_o_historico_antigos_uma_vez() {
        let pasta = tempfile::tempdir().unwrap();
        fs::write(pasta.path().join("config.json"), r#"{"pasta_rascunhos":"C:/x","url_catalogo":"http://127.0.0.1:1/i.json","mostrar_trailers":false}"#).unwrap();
        let antigo = Historico { entradas: vec![entrada("jak2-usa-ptbr", "1.0")] };
        fs::write(pasta.path().join("historico.json"), serde_json::to_string(&antigo).unwrap()).unwrap();

        let banco = Banco::abrir(pasta.path()).unwrap();
        assert_eq!(banco.preferencia("mostrar_trailers").as_deref(), Some("nao"));
        assert_eq!(banco.preferencia("url_catalogo").as_deref(), Some("http://127.0.0.1:1/i.json"));
        assert_eq!(banco.preferencia("pasta_rascunhos"), None, "a pasta de rascunhos não existe mais");
        assert_eq!(banco.historico(), antigo);
        drop(banco);

        // Na próxima abertura não traz de novo (não duplica o histórico).
        assert_eq!(Banco::abrir(pasta.path()).unwrap().historico().entradas.len(), 1);
    }

    #[test]
    fn json_antigo_invalido_e_ignorado() {
        let pasta = tempfile::tempdir().unwrap();
        fs::write(pasta.path().join("config.json"), "{ quebrado").unwrap();
        fs::write(pasta.path().join("historico.json"), "[]]").unwrap();
        let banco = Banco::abrir(pasta.path()).unwrap();
        assert_eq!(banco.preferencia("mostrar_trailers"), None);
        assert!(banco.historico().entradas.is_empty());
    }

    #[test]
    fn banco_corrompido_e_guardado_de_lado_e_comeca_um_novo() {
        let pasta = tempfile::tempdir().unwrap();
        fs::write(pasta.path().join(ARQUIVO_BANCO), "isto não é um banco SQLite, só texto qualquer").unwrap();
        let banco = Banco::abrir(pasta.path()).unwrap();
        banco.definir_preferencia("mostrar_trailers", "sim").unwrap();
        assert!(pasta.path().join(format!("{ARQUIVO_BANCO}.corrompido")).is_file());
    }
}
