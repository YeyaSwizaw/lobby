use std::net::{TcpStream, SocketAddr};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::io::{Write, Result};
use std::thread;

use vec_map::VecMap;

use serde::de::DeserializeOwned;

use serde_json::Deserializer;

use event::Event;

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn spawn<Data>(
        id: usize, 
        stream: TcpStream, 
        addr: SocketAddr, 
        connections: Arc<Mutex<VecMap<Connection>>>, 
        tx: Sender<Event<Data>>
    ) -> Connection
    where 
        Data: 'static + Send + DeserializeOwned 
    {
        tx.send(Event::connection_received(id, addr)).unwrap();

        let thread_stream = stream.try_clone().unwrap();
        thread_stream.set_nonblocking(false).unwrap();

        thread::spawn(move || {
            let mut errors = 0;
            
            loop {
                if errors > 10 {
                    break
                }

                let mut de = Deserializer::from_reader(thread_stream.try_clone().unwrap());
                match Data::deserialize(&mut de) {
                    Ok(data) => {
                        tx.send(Event::data_received(id, data)).unwrap();
                        errors = 0;
                    },

                    Err(ref e) if e.is_io() => {
                        break
                    },

                    Err(e) => {
                        tx.send(Event::data_error(id, e)).unwrap();
                        errors += 1;
                    }
                }
            }

            connections.lock().unwrap().remove(id);
            tx.send(Event::connection_lost(id, addr)).unwrap();
        });

        Connection { 
            stream 
        }
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize> {
        self.stream.write(data)
    }
}
