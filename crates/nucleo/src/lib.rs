//! Núcleo do Tradumix.

mod aplicar;
mod baixar;
mod catalogo;
mod cue;
mod erro;
mod historico;
mod instalar_pc;
mod iso9660;
mod pacote;
mod plataformas;
mod processo;
mod ps1;
mod ps2;
mod rede;
mod sincronizar;

pub use aplicar::{Etapa, Pedido, Resultado, aplicar, aplicar_com_rede, conferir, nome_saida};
pub use baixar::baixar_patch;
pub use catalogo::{FORMATO_INDICE, Indice, ler_indice};
pub use erro::Erro;
pub use historico::{Atualizacao, Entrada, Historico};
pub use instalar_pc::{RemocaoPc, instalado_pc, remover_pc};
pub use processo::GrupoDeProcessos;
pub use cue::{Cue, Faixa, TipoFaixa, escrever_cue, ler_cue};
pub use iso9660::{Iso, Setores};
pub use pacote::{
    ArquivoBaixavel, ArquivoDoPc, CargaPacotes, Estado, FORMATO_FICHA, Ficha, Imagem, Imagens, Instalacao, Jogo, Midia, NOME_CAPA_ACERVO, Pacote, PacoteInvalido,
    Tradutor, carregar_pacotes, comparar_versoes, e_novo,
};
pub use plataformas::{Alvo, InfoPlataforma, OrigemPc, OrigemPs1, OrigemPs2, PLATAFORMAS, Plataforma, ReferenciaPc, ResultadoPs1, ResultadoPs2, ler_alvo};
pub use rede::{Rede, RedeUreq};
pub use sincronizar::{Capas, Sincronizacao, sincronizar};
pub use ps1::{IdentidadePs1, hash_ra_ps1, identificar_bin_ps1, identificar_ps1};
pub use ps2::{IdentidadePs2, Regiao, hash_ra, identificar, regiao_do_serial, serial_do_executavel};
