use std::net::{TcpStream, SocketAddr};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::io::{Write, Result};
use std::thread;

use vec_map::VecMap;

use serde::de::DeserializeOwned;

use serde_json::Deserializer;

use message::Message;

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn spawn<Data>(
        id: usize, 
        stream: TcpStream, 
        addr: SocketAddr, 
        connections: Arc<Mutex<VecMap<Connection>>>, 
        tx: Sender<Message<Data>>
    ) -> Connection
    where 
        Data: 'static + Send + DeserializeOwned 
    {
        tx.send(Message::connection_received(id, addr)).unwrap();

        let thread_stream = stream.try_clone().unwrap();
        thread_stream.set_nonblocking(false).unwrap();

        thread::spawn(move || loop {
            let mut de = Deserializer::from_reader(thread_stream.try_clone().unwrap());
            match Data::deserialize(&mut de) {
                Ok(data) => tx.send(Message::data_received(id, data)).unwrap(),

                Err(ref e) if e.is_io() => {
                    connections.lock().unwrap().remove(id);
                    tx.send(Message::connection_lost(id, addr)).unwrap();
                    break
                },

                Err(e) => tx.send(Message::data_error(id, e)).unwrap()
            }
        });

        Connection { 
            stream 
        }
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize> {
        self.stream.write(data)
    }
}
