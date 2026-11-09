//! `tradumix-cli`: o núcleo do Tradumix na linha de comando, para a oficina testar uma tradução
//! antes de publicar. Toda resposta é um JSON na saída padrão; o progresso vai para a saída de erro.
//!
//! Código de saída: 0 = ok, 1 = recusado (o JSON traz `"ok": false` e `"erro"`), 2 = uso errado.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use nucleo::lista::{ListaJogos, SituacaoRa, hoje, montar_jogos};
use nucleo::{Etapa, Pedido, Sincronizacao, aplicar, carregar_pacotes, identificar, identificar_ps1, ler_pacote_da_pasta, nome_saida, remover_pc};
use serde_json::{Value, json};

const USO: &str = "uso:
  tradumix-cli validar <pasta-do-pacote>
  tradumix-cli identificar <arquivo> --plataforma ps1|ps2
  tradumix-cli instalar <pasta-do-pacote> --original <caminho> [--saida <caminho>] [--sobrescrever] --xdelta3 <exe>
  tradumix-cli remover <pasta-do-pacote> --pasta <pasta-do-jogo>
  tradumix-cli previa <pasta-de-pacotes> --saida <pasta>";

/// Como o comando terminou.
enum Fim {
    Ok(Value),
    Recusado(String),
    Uso(String),
}

/// Argumentos depois do comando: um posicional e opções `--nome valor` ou `--sinal`.
struct Args {
    posicional: Option<String>,
    opcoes: Vec<(String, Option<String>)>,
}

const SINAIS: &[&str] = &["sobrescrever"];

impl Args {
    fn ler(lista: &[String]) -> Result<Args, String> {
        let mut args = Args { posicional: None, opcoes: Vec::new() };
        let mut i = 0;
        while i < lista.len() {
            let a = &lista[i];
            if let Some(nome) = a.strip_prefix("--") {
                if SINAIS.contains(&nome) {
                    args.opcoes.push((nome.to_string(), None));
                } else {
                    let valor = lista.get(i + 1).ok_or(format!("falta o valor de --{nome}"))?;
                    args.opcoes.push((nome.to_string(), Some(valor.clone())));
                    i += 1;
                }
            } else if args.posicional.is_none() {
                args.posicional = Some(a.clone());
            } else {
                return Err(format!("argumento a mais: {a}"));
            }
            i += 1;
        }
        Ok(args)
    }

    fn posicional(&self, nome: &str) -> Result<PathBuf, String> {
        self.posicional.as_ref().map(PathBuf::from).ok_or(format!("falta {nome}"))
    }

    fn valor(&self, nome: &str) -> Option<&str> {
        self.opcoes.iter().find(|(n, _)| n == nome).and_then(|(_, v)| v.as_deref())
    }

    fn exigir(&self, nome: &str) -> Result<&str, String> {
        self.valor(nome).ok_or(format!("falta --{nome}"))
    }

    fn sinal(&self, nome: &str) -> bool {
        self.opcoes.iter().any(|(n, _)| n == nome)
    }
}

fn main() -> ExitCode {
    let lista: Vec<String> = std::env::args().skip(1).collect();
    let fim = match lista.split_first() {
        None => Fim::Uso("falta o comando".into()),
        Some((comando, resto)) => match Args::ler(resto) {
            Err(e) => Fim::Uso(e),
            Ok(args) => executar(comando, &args),
        },
    };
    let (codigo, json) = match fim {
        Fim::Ok(v) => (0, v),
        Fim::Recusado(erro) => (1, json!({ "ok": false, "erro": erro })),
        Fim::Uso(erro) => (2, json!({ "ok": false, "erro": format!("{erro}\n{USO}") })),
    };
    println!("{json}");
    ExitCode::from(codigo)
}

fn executar(comando: &str, args: &Args) -> Fim {
    let r = match comando {
        "validar" => validar(args),
        "identificar" => identificar_cmd(args),
        "instalar" => instalar(args),
        "remover" => remover(args),
        "previa" => previa(args),
        outro => return Fim::Uso(format!("comando desconhecido: {outro}")),
    };
    match r {
        Ok(v) => Fim::Ok(v),
        Err(Falha::Uso(e)) => Fim::Uso(e),
        Err(Falha::Recusa(e)) => Fim::Recusado(e),
    }
}

enum Falha {
    Uso(String),
    Recusa(String),
}

fn uso(e: String) -> Falha {
    Falha::Uso(e)
}

fn recusa(e: impl ToString) -> Falha {
    Falha::Recusa(e.to_string())
}

fn ra(alvo: &nucleo::Alvo) -> Option<SituacaoRa> {
    alvo.ra_compativel().map(|ok| if ok { SituacaoRa::Compativel } else { SituacaoRa::NaoReconhece })
}

fn validar(args: &Args) -> Result<Value, Falha> {
    let pasta = args.posicional("a pasta do pacote").map_err(uso)?;
    let pacote = ler_pacote_da_pasta(&pasta).map_err(recusa)?;
    pacote.conferir_baixados().map_err(recusa)?;
    Ok(json!({
        "ok": true,
        "id": pacote.ficha.id,
        "plataforma": pacote.ficha.plataforma,
        "ra": ra(&pacote.alvo),
        "contem_programa": pacote.ficha.contem_programa(),
    }))
}

fn identificar_cmd(args: &Args) -> Result<Value, Falha> {
    let arquivo = args.posicional("o arquivo").map_err(uso)?;
    match args.exigir("plataforma").map_err(uso)? {
        "ps1" => {
            let id = identificar_ps1(&arquivo).map_err(|e| recusa(e.mensagem()))?;
            Ok(json!({ "serial": id.serial, "regiao": id.regiao, "hash_ra": id.hash_ra, "tamanho": id.tamanho_dados }))
        }
        "ps2" => {
            let id = identificar(&arquivo).map_err(|e| recusa(e.mensagem()))?;
            Ok(json!({ "serial": id.serial, "regiao": id.regiao, "hash_ra": id.hash_ra, "tamanho": id.tamanho }))
        }
        outra => Err(uso(format!("plataforma sem identificação: {outra} (use ps1 ou ps2)"))),
    }
}

fn instalar(args: &Args) -> Result<Value, Falha> {
    let pasta = args.posicional("a pasta do pacote").map_err(uso)?;
    let original = PathBuf::from(args.exigir("original").map_err(uso)?);
    let xdelta3 = PathBuf::from(args.exigir("xdelta3").map_err(uso)?);
    let pacote = ler_pacote_da_pasta(&pasta).map_err(recusa)?;
    if !pacote.patch_disponivel() {
        return Err(recusa(format!("o arquivo {} não está na pasta do pacote", pacote.ficha.baixavel_principal().nome)));
    }
    let saida = args.valor("saida").map(PathBuf::from).unwrap_or_else(|| nome_saida(&pacote, &original));
    let pedido = Pedido { iso_original: original, saida, sobrescrever: args.sinal("sobrescrever"), xdelta3 };
    let mut ultimo: Option<(Etapa, u32)> = None;
    let mut progresso = |etapa: Etapa, fracao: f32| {
        let pct = (fracao * 100.0).round() as u32;
        if ultimo != Some((etapa, pct)) {
            ultimo = Some((etapa, pct));
            let nome = serde_json::to_value(etapa).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
            eprintln!("{nome} {pct}%");
        }
    };
    let resultado = aplicar(&pacote, &pedido, &mut progresso, &AtomicBool::new(false)).map_err(|e| recusa(e.mensagem()))?;
    serde_json::to_value(resultado).map_err(recusa)
}

fn remover(args: &Args) -> Result<Value, Falha> {
    let pasta = args.posicional("a pasta do pacote").map_err(uso)?;
    let jogo = PathBuf::from(args.exigir("pasta").map_err(uso)?);
    let pacote = ler_pacote_da_pasta(&pasta).map_err(recusa)?;
    let r = remover_pc(&pacote, &jogo).map_err(|e| recusa(e.mensagem()))?;
    serde_json::to_value(r).map_err(recusa)
}

fn previa(args: &Args) -> Result<Value, Falha> {
    let pasta = args.posicional("a pasta de pacotes").map_err(uso)?;
    let saida = PathBuf::from(args.exigir("saida").map_err(uso)?);
    let carga = carregar_pacotes(&pasta);
    if let Some(ruim) = carga.invalidos.first() {
        return Err(recusa(format!("pacote inválido em {}: {}", ruim.pasta.display(), ruim.motivo)));
    }
    let jogos = montar_jogos(&carga.pacotes, &hoje());
    let n = jogos.len();
    let lista = ListaJogos {
        jogos,
        atualizacoes: Vec::new(),
        catalogo: Sincronizacao { offline: false, atualizado_em: None, aviso: None },
        mostrar_trailers: true,
    };
    gravar_lista(&saida, &lista).map_err(recusa)?;
    Ok(json!({ "ok": true, "jogos": n }))
}

fn gravar_lista(saida: &Path, lista: &ListaJogos) -> Result<(), String> {
    std::fs::create_dir_all(saida).map_err(|e| e.to_string())?;
    let texto = serde_json::to_string_pretty(lista).map_err(|e| e.to_string())?;
    std::fs::write(saida.join("lista.json"), texto).map_err(|e| e.to_string())
}
