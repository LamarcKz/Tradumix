//! Servidor HTTP mínimo para os testes de rede: respostas por caminho, quedas simuladas
//! e contagem de requisições.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum Resposta {
    Ok(Vec<u8>),
    Status(u16),
    /// Anuncia o tamanho todo, manda só a metade e fecha a conexão.
    Cortar(Vec<u8>),
    /// Manda o corpo em 10 pedaços, com uma pausa de `ms` entre eles.
    Lento(Vec<u8>, u64),
    /// Aceita a conexão e nunca responde.
    Mudo,
}

#[derive(Clone)]
pub struct Servidor {
    pub porta: u16,
    rotas: Arc<Mutex<HashMap<String, Resposta>>>,
    contagem: Arc<Mutex<HashMap<String, usize>>>,
}

impl Servidor {
    pub fn novo() -> Servidor {
        let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
        let porta = ouvinte.local_addr().unwrap().port();
        let s = Servidor { porta, rotas: Default::default(), contagem: Default::default() };
        let copia = s.clone();
        std::thread::spawn(move || {
            for conexao in ouvinte.incoming().flatten() {
                let c = copia.clone();
                std::thread::spawn(move || c.atender(conexao));
            }
        });
        s
    }

    pub fn url(&self, caminho: &str) -> String {
        format!("http://127.0.0.1:{}{caminho}", self.porta)
    }

    pub fn definir(&self, caminho: &str, resposta: Resposta) {
        self.rotas.lock().unwrap().insert(caminho.to_string(), resposta);
    }

    pub fn pedidos(&self, caminho: &str) -> usize {
        *self.contagem.lock().unwrap().get(caminho).unwrap_or(&0)
    }

    fn atender(&self, mut conexao: TcpStream) {
        let mut leitor = BufReader::new(conexao.try_clone().unwrap());
        let mut linha = String::new();
        if leitor.read_line(&mut linha).is_err() {
            return;
        }
        let caminho = linha.split_whitespace().nth(1).unwrap_or("/").to_string();
        loop {
            let mut cabecalho = String::new();
            if leitor.read_line(&mut cabecalho).unwrap_or(0) == 0 || cabecalho == "\r\n" {
                break;
            }
        }
        *self.contagem.lock().unwrap().entry(caminho.clone()).or_default() += 1;
        let resposta = self.rotas.lock().unwrap().get(&caminho).cloned().unwrap_or(Resposta::Status(404));
        let _ = match resposta {
            Resposta::Ok(corpo) => {
                let _ = write!(conexao, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", corpo.len());
                conexao.write_all(&corpo)
            }
            Resposta::Status(codigo) => write!(conexao, "HTTP/1.1 {codigo} Erro\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"),
            Resposta::Lento(corpo, ms) => {
                let _ = write!(conexao, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", corpo.len());
                let pedaco = corpo.len().div_ceil(10).max(1);
                for parte in corpo.chunks(pedaco) {
                    let _ = conexao.write_all(parte);
                    let _ = conexao.flush();
                    std::thread::sleep(std::time::Duration::from_millis(ms));
                }
                Ok(())
            }
            Resposta::Mudo => {
                std::thread::sleep(std::time::Duration::from_secs(30));
                Ok(())
            }
            Resposta::Cortar(corpo) => {
                let _ = write!(conexao, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", corpo.len());
                conexao.write_all(&corpo[..corpo.len() / 2])
            }
        };
        let _ = conexao.flush();
        let _ = conexao.shutdown(std::net::Shutdown::Both);
    }
}
