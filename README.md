# lobby

Simple Rust TCP server, inspired by https://github.com/dradtke/lobby

Provides an iterator over 'messages', which are events caused by connected clients: either connections, disconnections, or received data.

```rust
let lobby = Lobby::spawn("127.0.0.1:8080").unwrap();

loop {
    for msg in lobby.messages() {
        lobby.send(&format!("{:?}", msg).as_bytes());
    }
}
```
