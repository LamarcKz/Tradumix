//! Núcleo do Tradumix.

mod aplicar;
mod baixar;
mod catalogo;
mod erro;
mod historico;
mod iso9660;
mod pacote;
mod processo;
mod ps2;
mod rede;
mod sincronizar;

pub use aplicar::{Etapa, Pedido, Resultado, aplicar, aplicar_com_rede, conferir, nome_saida};
pub use baixar::baixar_patch;
pub use catalogo::{FORMATO_INDICE, Indice, ler_indice};
pub use erro::Erro;
pub use historico::{Atualizacao, Entrada, Historico};
pub use processo::GrupoDeProcessos;
pub use iso9660::Iso;
pub use pacote::{
    ArquivoRemoto, Arquivos, Download, CargaPacotes, Estado, FORMATO_FICHA, Ficha, Imagem, Jogo, JogoNoCatalogo, Origem, Pacote, PacoteInvalido,
    ResultadoEsperado, Tradutor, agrupar_por_jogo, carregar_pacotes, comparar_versoes, serve_para,
};
pub use rede::{Rede, RedeUreq};
pub use sincronizar::{Sincronizacao, sincronizar};
pub use ps2::{IdentidadePs2, Regiao, hash_ra, identificar, regiao_do_serial, serial_do_executavel};
