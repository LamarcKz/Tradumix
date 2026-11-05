mod comum;

use std::time::{Duration, Instant};

use comum::servidor::{Resposta, Servidor};
use nucleo::{Rede, RedeUreq};

/// Um download grande e lento, mas que não para de chegar, não pode cair por um limite de tempo total.
#[test]
fn download_lento_mas_continuo_nao_cai() {
    let s = Servidor::novo();
    let corpo = vec![7u8; 100_000];
    s.definir("/lento", Resposta::Lento(corpo.clone(), 300));
    let rede = RedeUreq::com_tempos(Duration::from_secs(1), Duration::from_secs(1));
    let mut destino = Vec::new();
    let total = rede.baixar(&s.url("/lento"), &mut destino, 200_000, &mut |_| {}).unwrap();
    assert_eq!(total, 100_000);
    assert_eq!(destino, corpo);
}

/// Um servidor que aceita a conexão e não responde dá erro em vez de travar o app.
#[test]
fn servidor_mudo_da_erro_em_vez_de_travar() {
    let s = Servidor::novo();
    s.definir("/mudo", Resposta::Mudo);
    let rede = RedeUreq::com_tempos(Duration::from_secs(1), Duration::from_secs(1));
    let inicio = Instant::now();
    let r = rede.baixar(&s.url("/mudo"), &mut Vec::new(), 1000, &mut |_| {});
    assert!(r.is_err());
    assert!(inicio.elapsed() < Duration::from_secs(10), "demorou {:?}", inicio.elapsed());
}
