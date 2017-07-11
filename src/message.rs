use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Message {
    from: usize,
    message: MessageKind
}

#[derive(Debug, Clone)]
pub enum MessageKind {
    ConnectionReceived(SocketAddr),
    ConnectionLost(SocketAddr),
    DataReceived(String)
}

impl Message {
    pub(crate) fn connection_received(from: usize, addr: SocketAddr) -> Message {
        Message {
            from,
            message: MessageKind::ConnectionReceived(addr)
        }
    }

    pub(crate) fn connection_lost(from: usize, addr: SocketAddr) -> Message {
        Message {
            from,
            message: MessageKind::ConnectionLost(addr)
        }
    }

    pub(crate) fn data_received(from: usize, data: String) -> Message {
        Message {
            from,
            message: MessageKind::DataReceived(data)
        }
    }
}
