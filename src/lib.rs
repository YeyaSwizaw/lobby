#![feature(conservative_impl_trait)]

extern crate vec_map;

use std::io::{Error, Result};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::net::{ToSocketAddrs, TcpListener};
use std::thread;

use vec_map::VecMap;

use connection::Connection;

mod connection;
mod message;

pub use message::Message;

pub struct Lobby {
    listener_tx: Sender<()>,
    message_rx: Receiver<Message>,

    connections: Arc<Mutex<VecMap<Connection>>>
}

impl Lobby {
    pub fn spawn<A: ToSocketAddrs>(addr: A) -> Result<Lobby> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;

        let (listener_tx, rx) = channel();
        let (tx, message_rx) = channel();

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
            message_rx,
            connections
        })
    }

    pub fn messages<'a>(&'a self) -> impl Iterator<Item=Message> + 'a {
        self.message_rx.try_iter()
    }

    pub fn send_to_pred<P: Fn(usize) -> bool>(&self, pred: P, data: &[u8]) -> std::result::Result<(), Vec<(usize, Error)>> {
        let mut errors = Vec::new();

        for conn in self.connections.lock().unwrap().iter_mut() {
            if pred(conn.0) {
                if let Err(e) = conn.1.send(data) {
                    errors.push((conn.0, e));
                }
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    pub fn send_to_except(&self, except: usize, data: &[u8]) -> std::result::Result<(), Vec<(usize, Error)>> {
        self.send_to_pred(move |id| id != except, data)
    }

    pub fn send(&self, data: &[u8]) -> std::result::Result<(), Vec<(usize, Error)>> {
        self.send_to_pred(|_| true, data)
    }
}

impl Drop for Lobby {
    fn drop(&mut self) {
        self.listener_tx.send(()).unwrap();
    }
}
