use std::net::SocketAddr;

use serde_json::Error;

#[derive(Debug)]
pub struct Event<Data> {
    pub from: usize,
    pub event: EventKind<Data>
}

#[derive(Debug)]
pub enum EventKind<Data> {
    ConnectionReceived(SocketAddr),
    ConnectionLost(SocketAddr),
    DataReceived(Data),
    DataError(Error)
}

impl<Data> Event<Data> {
    pub(crate) fn connection_received(from: usize, addr: SocketAddr) -> Event<Data> {
        Event {
            from,
            event: EventKind::ConnectionReceived(addr)
        }
    }

    pub(crate) fn connection_lost(from: usize, addr: SocketAddr) -> Event<Data> {
        Event {
            from,
            event: EventKind::ConnectionLost(addr)
        }
    }

    pub(crate) fn data_received(from: usize, data: Data) -> Event<Data> {
        Event {
            from,
            event: EventKind::DataReceived(data)
        }
    }

    pub(crate) fn data_error(from: usize, error: Error) -> Event<Data> {
        Event {
            from,
            event: EventKind::DataError(error)
        }
    }
}
