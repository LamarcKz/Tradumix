use std::io;

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
        }
    }
}

impl From<io::Error> for Erro {
    fn from(e: io::Error) -> Self {
        Erro::Io(e.to_string())
    }
}
