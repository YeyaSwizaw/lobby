use std::net::{TcpStream, SocketAddr, Shutdown};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::io::{Write, BufRead, BufReader, ErrorKind, Result};
use std::thread;

use vec_map::VecMap;

#[derive(Debug, Clone)]
pub enum Message {
    ConnectionReceived(usize, SocketAddr),
    ConnectionLost(usize, SocketAddr),
    DataReceived(usize, String)
}

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn spawn(id: usize, stream: TcpStream, addr: SocketAddr, connections: Arc<Mutex<VecMap<Connection>>>, tx: Sender<Message>) -> Connection {
        tx.send(Message::ConnectionReceived(id, addr)).unwrap();

        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut buf = String::new();

        thread::spawn(move || loop {
            match reader.read_line(&mut buf) {
                Ok(len) if len > 0 => {
                    tx.send(Message::DataReceived(id, buf.clone())).unwrap();
                }

                Err(ref e) if e.kind() != ErrorKind::WouldBlock => {
                    connections.lock().unwrap().remove(id);
                    tx.send(Message::ConnectionLost(id, addr)).unwrap();
                    break
                },

                _ => ()
            }

            buf.clear();
        });

        Connection {
            stream,
        }
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize> {
        self.stream.write(data)
    }
}
