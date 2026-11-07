//! Núcleo do Tradumix.

mod aplicar;
mod baixar;
mod catalogo;
mod erro;
mod historico;
mod iso9660;
mod pacote;
mod plataformas;
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
    ArquivoBaixavel, CargaPacotes, Estado, FORMATO_FICHA, Ficha, Imagem, Imagens, Instalacao, Jogo, Midia, NOME_CAPA_ACERVO, Pacote, PacoteInvalido,
    Tradutor, carregar_pacotes, comparar_versoes, e_novo,
};
pub use plataformas::{Alvo, InfoPlataforma, OrigemPs2, PLATAFORMAS, Plataforma, ResultadoPs2, ler_alvo};
pub use rede::{Rede, RedeUreq};
pub use sincronizar::{Capas, Sincronizacao, sincronizar};
pub use ps2::{IdentidadePs2, Regiao, hash_ra, identificar, regiao_do_serial, serial_do_executavel};
