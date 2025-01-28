use tokio::net::{TcpListener, TcpStream};

pub enum TunnelSocket {
    Listening(TcpListener),
    Connected(TcpStream),
}
