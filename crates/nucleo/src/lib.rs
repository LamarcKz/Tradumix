//! Núcleo do Tradumix.

mod aplicar;
mod erro;
mod iso9660;
mod pacote;
mod ps2;

pub use aplicar::{Etapa, Pedido, Resultado, aplicar, conferir, nome_saida};
pub use erro::Erro;
pub use iso9660::Iso;
pub use pacote::{
    Arquivos, CargaPacotes, Estado, FORMATO_FICHA, Ficha, Imagem, Jogo, JogoNoCatalogo, Origem, Pacote, PacoteInvalido,
    ResultadoEsperado, Tradutor, agrupar_por_jogo, carregar_pacotes, comparar_versoes, serve_para,
};
pub use ps2::{IdentidadePs2, Regiao, hash_ra, identificar, regiao_do_serial, serial_do_executavel};
