use std::{
    sync::{
        mpsc::{channel, Sender, Receiver},
        Arc, Mutex,
    },
    io::{Error, self},
    net::{ToSocketAddrs, TcpListener},
    marker::PhantomData,
    thread,
};

use serde::ser::Serialize;
use serde::de::DeserializeOwned;

use serde_json::to_vec;

use vec_map::VecMap;
use crate::connection::Connection;

mod connection;
mod event;

pub use crate::event::{Event, EventKind};

struct LobbyCloser {
    tx: Sender<()>
}

pub struct LobbySender {
    connections: Arc<Mutex<VecMap<Connection>>>,

    _closer: Arc<Mutex<LobbyCloser>>,
}

pub struct LobbyReceiver<Data> {
    event_rx: Receiver<Event<Data>>,

    _closer: Arc<Mutex<LobbyCloser>>,
}

pub struct Lobby<Data>{
    _phantom: PhantomData<Data>
}

impl<Data> Lobby<Data> where Data: 'static + Send + DeserializeOwned {
    pub fn spawn<A: ToSocketAddrs>(addr: A) -> io::Result<(LobbySender, LobbyReceiver<Data>)>
        where Data: 'static + Send + DeserializeOwned 
    {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;

        let (listener_tx, rx) = channel();
        let (tx, event_rx) = channel();

        let connections = Arc::new(Mutex::new(VecMap::new()));
        let thread_connections = connections.clone();

        thread::spawn(move || {
            let connections = thread_connections;
            let mut id = 0;

            loop {
                if let Ok(()) = rx.try_recv() {
                    break
                }

                if let Ok((stream, addr)) = listener.accept() {
                    let conn = Connection::spawn(id, stream, addr, connections.clone(), tx.clone());
                    connections.lock().unwrap().insert(id, conn);
                    id += 1;
                }
            }
        });

        let _closer = Arc::new(Mutex::new(LobbyCloser { tx: listener_tx }));

        let sender = LobbySender {
            connections,
            _closer: _closer.clone()
        };

        let receiver = LobbyReceiver {
            event_rx,
            _closer
        };

        Ok((sender, receiver))
    }
}

impl<Data> LobbyReceiver<Data> where Data: 'static + Send + DeserializeOwned {
    pub fn events<'a>(&'a self) -> impl Iterator<Item=Event<Data>> + 'a {
        self.event_rx.try_iter()
    }
}

impl LobbySender {
    pub fn send_to_pred<D, P>(&self, pred: P, data: D) -> Result<(), Vec<(usize, Error)>>
    where 
        P: Fn(usize) -> bool,
        D: Serialize
    {
        let mut errors = Vec::new();
        let data = to_vec(&data).unwrap();

        for conn in self.connections.lock().unwrap().iter_mut() {
            if pred(conn.0) {
                if let Err(e) = conn.1.send(&data) {
                    errors.push((conn.0, e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn send_to_except<D>(&self, except: usize, data: D) -> Result<(), Vec<(usize, Error)>> where D: Serialize {
        self.send_to_pred(move |id| id != except, data)
    }

    pub fn send_to<D>(&self, to: usize, data: D) -> Result<(), Vec<(usize, Error)>> where D: Serialize {
        self.send_to_pred(move |id| id == to, data)
    }

    pub fn send<D>(&self, data: D) -> Result<(), Vec<(usize, Error)>> where D: Serialize {
        self.send_to_pred(|_| true, data)
    }
}

impl Drop for LobbyCloser {
    fn drop(&mut self) {
        self.tx.send(()).unwrap();
    }
}
