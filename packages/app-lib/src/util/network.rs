use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;

pub async fn tcp_listen_any_loopback() -> io::Result<TcpListener> {
    // IPv4 is tried first for the best compatibility and performance with most systems.
    // IPv6 is also tried in case IPv4 is not available. Resolving "localhost" is avoided
    // to prevent failures deriving from improper name resolution setup. Any available
    // ephemeral port is used to prevent conflicts with other services. This is all as per
    // RFC 8252's recommendations
    const ANY_LOOPBACK_SOCKET: &[SocketAddr] = &[
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0),
    ];

    TcpListener::bind(ANY_LOOPBACK_SOCKET).await
}
