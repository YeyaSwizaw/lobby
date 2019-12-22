extern crate vec_map;

extern crate serde_json;
extern crate serde;

use std::io::{Error, self};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::net::{ToSocketAddrs, TcpListener};
use std::thread;

use vec_map::VecMap;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;

use serde_json::to_vec;

use crate::connection::Connection;

mod connection;
mod event;

pub use crate::event::{Event, EventKind};

pub struct Lobby<Data> {
    listener_tx: Sender<()>,
    event_rx: Receiver<Event<Data>>,

    connections: Arc<Mutex<VecMap<Connection>>>
}

impl<Data> Lobby<Data> where Data: 'static + Send + DeserializeOwned {
    pub fn spawn<A: ToSocketAddrs>(addr: A) -> io::Result<Lobby<Data>> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;

        let (listener_tx, rx) = channel();
        let (tx, event_rx) = channel();

        let connections = Arc::new(Mutex::new(VecMap::new()));
        let thread_conns = connections.clone();

        thread::spawn(move || {
            let connections = thread_conns;
            let mut id = 0;

            loop {
                if let Ok(()) = rx.try_recv() {
                    break
                }

                if let Ok((stream, addr)) = listener.accept() {
                    let tx = tx.clone();
                    let conn = Connection::spawn(id, stream, addr, connections.clone(), tx);
                    connections.lock().unwrap().insert(id, conn);
                    id += 1;
                }
            }
        });

        Ok(Lobby {
            listener_tx,
            event_rx,
            connections
        })
    }

    pub fn events<'a>(&'a self) -> impl Iterator<Item=Event<Data>> + 'a {
        self.event_rx.try_iter()
    }

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

impl<Data> Drop for Lobby<Data> {
    fn drop(&mut self) {
        self.listener_tx.send(()).unwrap();
    }
}
