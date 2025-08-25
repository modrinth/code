use crate::Result;
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

#[cfg(windows)]
pub async fn is_network_metered() -> Result<bool> {
    use windows::Networking::Connectivity::{
        NetworkCostType, NetworkInformation,
    };

    let cost_type = NetworkInformation::GetInternetConnectionProfile()?
        .GetConnectionCost()?
        .NetworkCostType()?;
    Ok(matches!(
        cost_type,
        NetworkCostType::Fixed | NetworkCostType::Variable
    ))
}

#[cfg(target_os = "macos")]
pub async fn is_network_metered() -> Result<bool> {
    use crate::ErrorKind;
    use cidre::dispatch::Queue;
    use cidre::nw::PathMonitor;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio_util::future::FutureExt;

    let (sender, mut receiver) = mpsc::channel(1);

    let queue = Queue::new();
    let mut monitor = PathMonitor::new();
    monitor.set_queue(&queue);
    monitor.set_update_handler(move |path| {
        let _ = sender.try_send(path.is_constrained() || path.is_expensive());
    });

    monitor.start();
    let result = receiver
        .recv()
        .timeout(Duration::from_millis(100))
        .await
        .ok()
        .flatten();
    monitor.cancel();

    result.ok_or_else(|| {
        ErrorKind::OtherError(
            "NWPathMonitor didn't provide an NWPath in time".to_string(),
        )
        .into()
    })
}

#[cfg(target_os = "linux")]
pub async fn is_network_metered() -> Result<bool> {
    // Thanks to https://github.com/Hakanbaban53/rclone-manager for showing how to do this
    use zbus::{Connection, Proxy};

    let connection = Connection::system().await?;
    let proxy = Proxy::new(
        &connection,
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        "org.freedesktop.NetworkManager",
    )
    .await?;
    let metered = proxy.get_property("Metered").await?;
    Ok(matches!(metered, 1 | 3))
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub async fn is_network_metered() -> Result<bool> {
    tracing::warn!(
        "is_network_metered called on unsupported platform. Assuming unmetered."
    );
    Ok(false)
}
