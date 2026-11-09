//! Instalar e remover uma tradução de PC na pasta do jogo.
//!
//! Tudo o que a tradução faz fica registrado em `<pasta do jogo>/.tradumix/<id>/`:
//! - `instalado.json`: versão e arquivos instalados (com o SHA-256 e se havia um original);
//! - `originais/<caminho>`: o arquivo do jogo que a tradução trocou.
//!
//! O `.zip` é extraído primeiro para `.tradumix/<id>/novo/` e conferido inteiro (lista e SHA-256)
//! antes de qualquer arquivo do jogo ser tocado. Na cópia, cada arquivo que vai ser trocado é
//! guardado em `desfazer/`; em qualquer erro ou cancelamento, tudo volta como estava.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::plataformas::pc::conferir_caminho;
use crate::rede::sha256_arquivo;
use crate::{ArquivoDoPc, Erro, Etapa, Instalacao, Pacote};

const PASTA_REGISTRO: &str = ".tradumix";

/// Programas que já existem na pasta do jogo e que a tradução nunca substitui.
const PROTEGIDOS: &[&str] = &["exe", "dll"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Instalado {
    id: String,
    versao: String,
    arquivos: Vec<ArquivoInstalado>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ArquivoInstalado {
    caminho: String,
    sha256: String,
    /// Havia um arquivo do jogo nesse lugar, guardado em `originais/`.
    original: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemocaoPc {
    pub removidos: usize,
    pub restaurados: usize,
    /// Arquivos que a loja trocou depois da instalação: ficaram como a loja deixou.
    pub mantidos: Vec<String>,
}

/// O caminho da ficha (com `/`) dentro de `pasta`.
pub(crate) fn no_jogo(pasta: &Path, caminho: &str) -> PathBuf {
    caminho.split('/').fold(pasta.to_path_buf(), |p, parte| p.join(parte))
}

fn registro(pasta: &Path, id: &str) -> PathBuf {
    pasta.join(PASTA_REGISTRO).join(id)
}

fn ler_instalado(reg: &Path) -> Option<Instalado> {
    serde_json::from_str(&fs::read_to_string(reg.join("instalado.json")).ok()?).ok()
}

fn arquivos_da_ficha(pacote: &Pacote) -> Result<&[ArquivoDoPc], Erro> {
    match &pacote.ficha.instalacao {
        Instalacao::PastaDoJogo { arquivos, .. } => Ok(arquivos),
        _ => Err(Erro::Io("esta tradução não é instalada na pasta do jogo".into())),
    }
}

/// A versão da tradução instalada nesta pasta, se houver.
pub fn instalado_pc(pacote: &Pacote, pasta: &Path) -> Option<String> {
    ler_instalado(&registro(pasta, &pacote.ficha.id)).map(|i| i.versao)
}

fn sha(caminho: &Path) -> Option<String> {
    sha256_arquivo(caminho).map(|s| s.to_lowercase())
}

fn copiar(de: &Path, para: &Path) -> io::Result<()> {
    if let Some(pai) = para.parent() {
        fs::create_dir_all(pai)?;
    }
    fs::copy(de, para).map(|_| ())
}

/// Apaga as pastas que ficaram vazias de `arquivo` para cima, sem passar de `limite`.
fn limpar_pastas_vazias(arquivo: &Path, limite: &Path) {
    let mut atual = arquivo.parent();
    while let Some(p) = atual {
        if p == limite || !p.starts_with(limite) || fs::remove_dir(p).is_err() {
            break;
        }
        atual = p.parent();
    }
}

/// Extrai o `.zip` em `destino`, aceitando só os arquivos da lista, cada um com o tamanho e o
/// SHA-256 da ficha. Entradas com caminho fora da regra são recusadas antes de gravar.
fn extrair(zip: &Path, destino: &Path, arquivos: &[ArquivoDoPc]) -> Result<(), Erro> {
    let ruim = |m: String| Erro::DownloadInvalido(m);
    let esperados: HashMap<&str, &ArquivoDoPc> = arquivos.iter().map(|a| (a.caminho.as_str(), a)).collect();
    let mut arquivo = zip::ZipArchive::new(File::open(zip)?).map_err(|e| ruim(format!("o pacote não é um .zip válido: {e}")))?;
    let mut vistos = HashSet::new();
    for i in 0..arquivo.len() {
        let mut entrada = arquivo.by_index(i).map_err(|e| ruim(format!("entrada do .zip ilegível: {e}")))?;
        if entrada.is_dir() {
            continue;
        }
        let nome = entrada.name().to_string();
        conferir_caminho(&nome).map_err(ruim)?;
        let Some(esperado) = esperados.get(nome.as_str()) else {
            return Err(ruim(format!("o pacote tem um arquivo que não está na ficha: {nome}")));
        };
        if !vistos.insert(nome.clone()) {
            return Err(ruim(format!("arquivo repetido no pacote: {nome}")));
        }
        let caminho = no_jogo(destino, &nome);
        if let Some(pai) = caminho.parent() {
            fs::create_dir_all(pai)?;
        }
        let mut saida = File::create(&caminho)?;
        // Lê no máximo um byte a mais que o declarado, para não encher o disco com um .zip falso.
        let copiados = io::copy(&mut (&mut entrada).take(esperado.tamanho + 1), &mut saida)?;
        if copiados != esperado.tamanho {
            return Err(ruim(format!("{nome} tem tamanho diferente do da ficha")));
        }
        drop(saida);
        if sha(&caminho).as_deref() != Some(esperado.sha256.to_lowercase().as_str()) {
            return Err(ruim(format!("{nome} veio diferente do da ficha (SHA-256)")));
        }
    }
    if let Some(falta) = arquivos.iter().find(|a| !vistos.contains(&a.caminho)) {
        return Err(ruim(format!("o pacote não tem o arquivo {}", falta.caminho)));
    }
    Ok(())
}

/// O que foi feito na pasta do jogo, para desfazer em caso de erro.
enum Feito {
    /// O arquivo não existia: desfazer é apagar.
    Criado(PathBuf),
    /// O arquivo existia e foi guardado em `desfazer/`: desfazer é copiar de volta.
    Trocado { destino: PathBuf, copia: PathBuf },
    /// O arquivo foi apagado (saiu da versão nova) e guardado em `desfazer/`.
    Apagado { destino: PathBuf, copia: PathBuf },
}

fn desfazer(feitos: &[Feito], pasta: &Path) {
    for f in feitos.iter().rev() {
        match f {
            Feito::Criado(p) => {
                let _ = fs::remove_file(p);
                limpar_pastas_vazias(p, pasta);
            }
            Feito::Trocado { destino, copia } | Feito::Apagado { destino, copia } => {
                let _ = copiar(copia, destino);
            }
        }
    }
}

/// Instala o pacote (já baixado) na pasta do jogo. Informa Aplicando e Verificando.
pub(crate) fn instalar_pc(
    pacote: &Pacote,
    pasta: &Path,
    progresso: &mut dyn FnMut(Etapa, f32),
    cancelar: &AtomicBool,
) -> Result<(), Erro> {
    let arquivos = arquivos_da_ficha(pacote)?;
    let id = &pacote.ficha.id;
    let reg = registro(pasta, id);
    let anterior = ler_instalado(&reg);
    let criou_registro = !reg.exists();
    let novo = reg.join("novo");
    let copias = reg.join("desfazer");
    let _ = fs::remove_dir_all(&novo);
    let _ = fs::remove_dir_all(&copias);

    let resultado = (|| -> Result<(), Erro> {
        progresso(Etapa::Aplicando, 0.0);
        fs::create_dir_all(&novo)?;
        extrair(&pacote.patch(), &novo, arquivos)?;
        let anteriores: HashMap<&str, &ArquivoInstalado> =
            anterior.iter().flat_map(|i| i.arquivos.iter()).map(|a| (a.caminho.as_str(), a)).collect();

        // Antes de tocar em qualquer coisa: programas do jogo não são substituídos.
        for a in arquivos {
            let destino = no_jogo(pasta, &a.caminho);
            let ext = a.caminho.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
            let do_jogo = destino.exists() && !anteriores.get(a.caminho.as_str()).is_some_and(|x| !x.original);
            if do_jogo && PROTEGIDOS.iter().any(|p| ext.eq_ignore_ascii_case(p)) {
                return Err(Erro::ArquivoDoJogoProtegido(a.caminho.clone()));
            }
        }

        let mut feitos: Vec<Feito> = Vec::new();
        let mut novos_originais: Vec<PathBuf> = Vec::new();
        let r = (|| -> Result<Vec<ArquivoInstalado>, Erro> {
            let mut registrados = Vec::new();
            let total = arquivos.len().max(1) as f32;
            for (n, a) in arquivos.iter().enumerate() {
                if cancelar.load(Ordering::Relaxed) {
                    return Err(Erro::Cancelado);
                }
                let destino = no_jogo(pasta, &a.caminho);
                let guardado = no_jogo(&reg.join("originais"), &a.caminho);
                let antes = anteriores.get(a.caminho.as_str());
                let atual = if destino.is_file() { sha(&destino) } else { None };
                let original = match (antes, &atual) {
                    // Ainda é o arquivo que a tradução pôs: o original guardado continua valendo.
                    (Some(x), Some(s)) if s.eq_ignore_ascii_case(&x.sha256) => x.original,
                    (Some(x), None) => x.original,
                    // Arquivo do jogo (ou trocado pela loja depois): passa a ser o original.
                    (_, Some(_)) => {
                        if guardado.exists() {
                            let copia = no_jogo(&copias.join("originais"), &a.caminho);
                            copiar(&guardado, &copia)?;
                            feitos.push(Feito::Trocado { destino: guardado.clone(), copia });
                        } else {
                            novos_originais.push(guardado.clone());
                        }
                        copiar(&destino, &guardado)?;
                        true
                    }
                    (None, None) => false,
                };
                if destino.exists() {
                    let copia = no_jogo(&copias, &a.caminho);
                    copiar(&destino, &copia)?;
                    feitos.push(Feito::Trocado { destino: destino.clone(), copia });
                } else {
                    feitos.push(Feito::Criado(destino.clone()));
                }
                copiar(&no_jogo(&novo, &a.caminho), &destino)?;
                registrados.push(ArquivoInstalado { caminho: a.caminho.clone(), sha256: a.sha256.to_lowercase(), original });
                progresso(Etapa::Aplicando, (n + 1) as f32 / total);
            }

            // Arquivos da versão anterior que saíram da nova: voltam ao original ou são apagados.
            let atuais: HashSet<&str> = arquivos.iter().map(|a| a.caminho.as_str()).collect();
            for velho in anterior.iter().flat_map(|i| i.arquivos.iter()).filter(|v| !atuais.contains(v.caminho.as_str())) {
                let destino = no_jogo(pasta, &velho.caminho);
                let ainda_nosso = sha(&destino).is_some_and(|s| s.eq_ignore_ascii_case(&velho.sha256));
                if !ainda_nosso {
                    continue;
                }
                let copia = no_jogo(&copias, &velho.caminho);
                copiar(&destino, &copia)?;
                feitos.push(Feito::Apagado { destino: destino.clone(), copia });
                if velho.original {
                    copiar(&no_jogo(&reg.join("originais"), &velho.caminho), &destino)?;
                } else {
                    fs::remove_file(&destino)?;
                    limpar_pastas_vazias(&destino, pasta);
                }
            }

            progresso(Etapa::Verificando, 0.0);
            for a in arquivos {
                if sha(&no_jogo(pasta, &a.caminho)).as_deref() != Some(a.sha256.to_lowercase().as_str()) {
                    return Err(Erro::Io(format!("o arquivo {} não ficou igual ao do pacote", a.caminho)));
                }
            }
            Ok(registrados)
        })();

        match r {
            Ok(registrados) => {
                // Originais de arquivos que saíram da versão nova não servem mais.
                if let Some(ant) = &anterior {
                    for velho in ant.arquivos.iter().filter(|v| v.original && !registrados.iter().any(|r| r.caminho == v.caminho)) {
                        let g = no_jogo(&reg.join("originais"), &velho.caminho);
                        let _ = fs::remove_file(&g);
                        limpar_pastas_vazias(&g, &reg);
                    }
                }
                let instalado = Instalado { id: id.clone(), versao: pacote.ficha.versao.clone(), arquivos: registrados };
                let texto = serde_json::to_string_pretty(&instalado).map_err(|e| Erro::Io(e.to_string()))?;
                fs::write(reg.join("instalado.json"), texto)?;
                progresso(Etapa::Verificando, 1.0);
                Ok(())
            }
            Err(e) => {
                desfazer(&feitos, pasta);
                // Originais guardados só nesta tentativa não servem mais.
                for g in &novos_originais {
                    let _ = fs::remove_file(g);
                    limpar_pastas_vazias(g, &reg);
                }
                Err(e)
            }
        }
    })();

    let _ = fs::remove_dir_all(&novo);
    let _ = fs::remove_dir_all(&copias);
    if resultado.is_err() && criou_registro {
        let _ = fs::remove_dir_all(&reg);
        let _ = fs::remove_dir(pasta.join(PASTA_REGISTRO));
    }
    resultado
}

/// Tira a tradução: arquivos novos são apagados e os trocados voltam ao original, mas só se
/// ainda forem os que a tradução instalou. Os que a loja trocou depois ficam como estão.
pub fn remover_pc(pacote: &Pacote, pasta: &Path) -> Result<RemocaoPc, Erro> {
    let reg = registro(pasta, &pacote.ficha.id);
    let instalado = ler_instalado(&reg).ok_or(Erro::TraducaoNaoInstalada)?;
    let mut r = RemocaoPc { removidos: 0, restaurados: 0, mantidos: Vec::new() };
    for a in &instalado.arquivos {
        let destino = no_jogo(pasta, &a.caminho);
        let guardado = no_jogo(&reg.join("originais"), &a.caminho);
        match sha(&destino) {
            Some(s) if s.eq_ignore_ascii_case(&a.sha256) => {
                if a.original {
                    copiar(&guardado, &destino)?;
                    r.restaurados += 1;
                } else {
                    fs::remove_file(&destino)?;
                    limpar_pastas_vazias(&destino, pasta);
                    r.removidos += 1;
                }
            }
            Some(_) => r.mantidos.push(a.caminho.clone()),
            // Sumiu da pasta: se havia um original, ele volta.
            None if a.original => {
                copiar(&guardado, &destino)?;
                r.restaurados += 1;
            }
            None => {}
        }
    }
    fs::remove_dir_all(&reg)?;
    let _ = fs::remove_dir(pasta.join(PASTA_REGISTRO));
    Ok(r)
}
