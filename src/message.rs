use std::net::SocketAddr;

use serde_json::Error;

#[derive(Debug)]
pub struct Message<Data> {
    pub from: usize,
    pub message: MessageKind<Data>
}

#[derive(Debug)]
pub enum MessageKind<Data> {
    ConnectionReceived(SocketAddr),
    ConnectionLost(SocketAddr),
    DataReceived(Data),
    DataError(Error)
}

impl<Data> Message<Data> {
    pub(crate) fn connection_received(from: usize, addr: SocketAddr) -> Message<Data> {
        Message {
            from,
            message: MessageKind::ConnectionReceived(addr)
        }
    }

    pub(crate) fn connection_lost(from: usize, addr: SocketAddr) -> Message<Data> {
        Message {
            from,
            message: MessageKind::ConnectionLost(addr)
        }
    }

    pub(crate) fn data_received(from: usize, data: Data) -> Message<Data> {
        Message {
            from,
            message: MessageKind::DataReceived(data)
        }
    }

    pub(crate) fn data_error(from: usize, error: Error) -> Message<Data> {
        Message {
            from,
            message: MessageKind::DataError(error)
        }
    }
}
