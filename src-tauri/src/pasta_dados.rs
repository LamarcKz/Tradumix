//! Onde ficam os dados do app (o banco `tradumix.db` e o catálogo baixado), como no PlayTrove:
//! - app instalado: a pasta padrão do Windows (`%APPDATA%\io.github.tradumix`);
//! - versão portátil (o `.zip`, com `portable.txt` ao lado do `.exe`): a pasta `dados` ali do lado;
//! - `npm run dev`: uma cópia da pasta do app instalado, ao lado dela (`... Dev`), para o código em
//!   desenvolvimento nunca mexer nos dados de verdade.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// O arquivo que deixa o app portátil: vem no `.zip`, ao lado do `Tradumix.exe`.
const MARCADOR_PORTATIL: &str = "portable.txt";

/// A pasta dos dados da versão portátil, ao lado do `.exe`.
const PASTA_PORTATIL: &str = "dados";

/// A pasta de dados desta vez. `padrao` é a pasta do app instalado; `desenvolvimento` é o
/// `npm run dev` (build de debug), que usa a cópia ao lado e nunca é portátil.
pub fn pasta_de_dados(exe: &Path, padrao: PathBuf, desenvolvimento: bool) -> PathBuf {
    if desenvolvimento {
        return pasta_de_desenvolvimento(&padrao);
    }
    pasta_portatil(exe).unwrap_or(padrao)
}

/// É a versão portátil? Nunca no modo de desenvolvimento.
pub fn e_portatil(exe: &Path, desenvolvimento: bool) -> bool {
    !desenvolvimento && pasta_portatil(exe).is_some()
}

/// A cópia do `npm run dev`: a pasta do app instalado com " Dev" no fim do nome, do lado dela.
pub fn pasta_de_desenvolvimento(real: &Path) -> PathBuf {
    let nome = real.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    real.with_file_name(format!("{nome} Dev"))
}

/// Na primeira vez (ou depois do `npm run dev:nova-copia`), copia os dados do app instalado para a
/// pasta do modo de desenvolvimento. A pasta de verdade só é lida. Sem ela, a cópia começa vazia.
pub fn preparar_copia_de_desenvolvimento(real: &Path, dev: &Path) -> io::Result<()> {
    if dev.exists() {
        return Ok(());
    }
    if real.is_dir() {
        copiar_pasta(real, dev)?;
        copiar_banco(real, dev)?;
    }
    fs::create_dir_all(dev)
}

/// O banco vai por `VACUUM INTO` (como no PlayTrove): grava uma cópia inteira e coerente mesmo com o
/// app instalado aberto e gravando, coisa que copiar o arquivo não garante.
fn copiar_banco(real: &Path, dev: &Path) -> io::Result<()> {
    let origem = real.join(crate::banco::ARQUIVO_BANCO);
    if !origem.is_file() {
        return Ok(());
    }
    let erro = |e: rusqlite::Error| io::Error::other(e.to_string());
    let banco = rusqlite::Connection::open_with_flags(&origem, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(erro)?;
    let destino = dev.join(crate::banco::ARQUIVO_BANCO);
    banco.execute("VACUUM INTO ?1", [destino.to_string_lossy()]).map_err(erro)?;
    Ok(())
}

fn pasta_portatil(exe: &Path) -> Option<PathBuf> {
    let pasta = exe.parent()?;
    pasta.join(MARCADOR_PORTATIL).is_file().then(|| pasta.join(PASTA_PORTATIL))
}

/// Copia a pasta inteira, menos os downloads pela metade (`.parcial`), que não servem para nada, e o
/// banco com os arquivos do SQLite ao lado dele (o banco vai à parte, por `copiar_banco`).
fn copiar_pasta(de: &Path, para: &Path) -> io::Result<()> {
    fs::create_dir_all(para)?;
    for item in fs::read_dir(de)? {
        let item = item?;
        let origem = item.path();
        let destino = para.join(item.file_name());
        if item.file_type()?.is_dir() {
            copiar_pasta(&origem, &destino)?;
        } else if origem.extension().is_none_or(|e| e != "parcial")
            && !item.file_name().to_string_lossy().starts_with(crate::banco::ARQUIVO_BANCO)
        {
            fs::copy(&origem, &destino)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn exe_em(pasta: &Path) -> PathBuf {
        pasta.join("Tradumix.exe")
    }

    #[test]
    fn sem_marcador_usa_a_pasta_padrao() {
        let pasta = tempfile::tempdir().unwrap();
        let padrao = pasta.path().join("instalado");
        assert_eq!(pasta_de_dados(&exe_em(pasta.path()), padrao.clone(), false), padrao);
        assert!(!e_portatil(&exe_em(pasta.path()), false));
    }

    #[test]
    fn com_portable_txt_usa_a_pasta_dados_ao_lado() {
        let pasta = tempfile::tempdir().unwrap();
        fs::write(pasta.path().join("portable.txt"), "").unwrap();
        let padrao = pasta.path().join("instalado");
        assert_eq!(pasta_de_dados(&exe_em(pasta.path()), padrao, false), pasta.path().join("dados"));
        assert!(e_portatil(&exe_em(pasta.path()), false));
    }

    #[test]
    fn modo_de_desenvolvimento_nunca_e_portatil() {
        let pasta = tempfile::tempdir().unwrap();
        fs::write(pasta.path().join("portable.txt"), "").unwrap();
        let padrao = pasta.path().join("io.github.tradumix");
        assert_eq!(
            pasta_de_dados(&exe_em(pasta.path()), padrao, true),
            pasta.path().join("io.github.tradumix Dev")
        );
        assert!(!e_portatil(&exe_em(pasta.path()), true));
    }

    #[test]
    fn portatil_e_instalado_nao_dividem_a_pasta() {
        let pasta = tempfile::tempdir().unwrap();
        let padrao = pasta.path().join("instalado");
        let instalado = pasta_de_dados(&exe_em(pasta.path()), padrao.clone(), false);
        fs::write(pasta.path().join("portable.txt"), "").unwrap();
        let portatil = pasta_de_dados(&exe_em(pasta.path()), padrao.clone(), false);
        let dev = pasta_de_dados(&exe_em(pasta.path()), padrao, true);
        assert_ne!(instalado, portatil);
        assert_ne!(instalado, dev);
        assert_ne!(portatil, dev);
    }

    #[test]
    fn desenvolvimento_copia_os_dados_sem_mexer_nos_de_verdade() {
        let pasta = tempfile::tempdir().unwrap();
        let real = pasta.path().join("io.github.tradumix");
        fs::create_dir_all(real.join("catalogo/jak2")).unwrap();
        fs::write(real.join("config.json"), "{\"mostrar_trailers\":false}").unwrap();
        fs::write(real.join("catalogo/jak2/ficha.json"), "{}").unwrap();
        fs::write(real.join("catalogo/jak2/patch.xdelta.parcial"), "pela metade").unwrap();
        let dev = pasta_de_desenvolvimento(&real);

        preparar_copia_de_desenvolvimento(&real, &dev).unwrap();
        assert_eq!(fs::read_to_string(dev.join("config.json")).unwrap(), "{\"mostrar_trailers\":false}");
        assert!(dev.join("catalogo/jak2/ficha.json").is_file());
        assert!(!dev.join("catalogo/jak2/patch.xdelta.parcial").exists());

        // Depois da primeira cópia, o dev é independente: mudar um não muda o outro.
        fs::write(dev.join("config.json"), "{}").unwrap();
        preparar_copia_de_desenvolvimento(&real, &dev).unwrap();
        assert_eq!(fs::read_to_string(dev.join("config.json")).unwrap(), "{}");
        assert_eq!(fs::read_to_string(real.join("config.json")).unwrap(), "{\"mostrar_trailers\":false}");
    }

    #[test]
    fn desenvolvimento_sem_app_instalado_comeca_vazio() {
        let pasta = tempfile::tempdir().unwrap();
        let real = pasta.path().join("io.github.tradumix");
        let dev = pasta_de_desenvolvimento(&real);
        preparar_copia_de_desenvolvimento(&real, &dev).unwrap();
        assert!(dev.is_dir());
        assert!(!real.exists());
    }

    #[test]
    fn desenvolvimento_copia_o_banco_inteiro_e_sem_o_diario_do_sqlite() {
        let pasta = tempfile::tempdir().unwrap();
        let real = pasta.path().join("io.github.tradumix");
        crate::banco::Banco::abrir(&real).unwrap().definir_preferencia("mostrar_trailers", "nao").unwrap();
        // O app instalado aberto, no meio de uma gravação ainda não salva: o SQLite deixa o diário
        // (tradumix.db-journal) ao lado do banco enquanto isso.
        let instalado = rusqlite::Connection::open(real.join("tradumix.db")).unwrap();
        instalado.execute_batch("BEGIN; UPDATE preferencias SET valor = 'sim' WHERE chave = 'mostrar_trailers';").unwrap();
        assert!(real.join("tradumix.db-journal").exists());
        let dev = pasta_de_desenvolvimento(&real);

        preparar_copia_de_desenvolvimento(&real, &dev).unwrap();
        assert!(!dev.join("tradumix.db-journal").exists(), "o diário não pode ir junto");
        let copia = crate::banco::Banco::abrir(&dev).unwrap();
        assert_eq!(copia.preferencia("mostrar_trailers").as_deref(), Some("nao"), "só o que já estava salvo");
        drop(instalado);
    }
}
