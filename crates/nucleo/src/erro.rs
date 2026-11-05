use std::io;
use std::path::PathBuf;

fn gb(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / 1_073_741_824.0).replace('.', ",")
}

/// Erros do núcleo. `mensagem` devolve o texto mostrado ao usuário.
#[derive(Debug, thiserror::Error)]
pub enum Erro {
    #[error("erro de arquivo: {0}")]
    Io(String),
    #[error("o arquivo não é uma ISO válida")]
    IsoInvalida,
    #[error("arquivo não encontrado na ISO: {0}")]
    ArquivoNaoEncontrado(String),
    #[error("a ISO não é de um jogo de PS2")]
    NaoEPs2,
    #[error("ISO de outro jogo: esperado {esperado_serial}, encontrado {encontrado_serial}")]
    JogoErrado { esperado_nome: String, esperado_serial: String, encontrado_serial: String },
    #[error("tamanho diferente: esperado {esperado}, encontrado {encontrado}")]
    TamanhoDiferente { esperado: u64, encontrado: u64 },
    #[error("o arquivo de saída já existe: {0}")]
    SaidaJaExiste(PathBuf),
    #[error("a saída é a própria ISO original")]
    SaidaIgualOriginal,
    #[error("espaço insuficiente: precisa {precisa}, livre {livre}")]
    EspacoInsuficiente { precisa: u64, livre: u64 },
    #[error("o xdelta3 falhou: {0}")]
    XdeltaFalhou(String),
    #[error("cancelado")]
    Cancelado,
    #[error("a ISO já está traduzida")]
    JaTraduzida,
    #[error("mesmo jogo ({serial}), outra versão do disco")]
    VersaoDiferente { esperado_nome: String, serial: String },
}

impl Erro {
    pub fn mensagem(&self) -> String {
        match self {
            Erro::Io(e) => format!("Não foi possível ler ou gravar o arquivo: {e}"),
            Erro::IsoInvalida => "O arquivo escolhido não é uma ISO válida.".into(),
            Erro::ArquivoNaoEncontrado(nome) => {
                format!("A ISO não tem o arquivo {nome}. Confira se é o jogo certo.")
            }
            Erro::NaoEPs2 => "Esta ISO não parece ser de um jogo de PlayStation 2.".into(),
            Erro::JogoErrado { esperado_nome, esperado_serial, encontrado_serial } => format!(
                "Esta tradução precisa da {esperado_nome}, serial {esperado_serial}. A ISO escolhida é {encontrado_serial}."
            ),
            Erro::TamanhoDiferente { .. } => {
                "A ISO é do jogo certo, mas é uma cópia diferente da esperada (o tamanho não confere). Use um dump sem alterações.".into()
            }
            Erro::SaidaJaExiste(caminho) => format!("Já existe um arquivo em {}.", caminho.display()),
            Erro::SaidaIgualOriginal => "A ISO traduzida não pode ser gravada por cima da original.".into(),
            Erro::EspacoInsuficiente { precisa, livre } => format!(
                "Falta espaço no disco: são necessários {} e há {} livres.",
                gb(*precisa),
                gb(*livre)
            ),
            Erro::XdeltaFalhou(detalhe) => format!(
                "Não foi possível aplicar a tradução. A ISO pode ser um dump diferente ou estar danificada. Detalhe: {detalhe}"
            ),
            Erro::Cancelado => "Tradução cancelada.".into(),
            Erro::VersaoDiferente { esperado_nome, serial } => format!(
                "Esta ISO é do jogo certo ({serial}), mas de outra versão do disco: o executável é diferente do usado na tradução. Esta tradução precisa da {esperado_nome} original."
            ),
            Erro::JaTraduzida => {
                "Esta ISO já está com esta tradução. Para gerar de novo, selecione a ISO original, sem tradução.".into()
            }
        }
    }
}

impl From<io::Error> for Erro {
    fn from(e: io::Error) -> Self {
        Erro::Io(e.to_string())
    }
}
