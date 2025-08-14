use crate::state::FriendsSocket;
use crate::state::friends::{TunnelSockets, WriteSocket};
use ariadne::networking::message::ClientToServerMessage;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;
use uuid::Uuid;

pub(super) enum InternalTunnelSocket {
    Listening(SocketAddr),
    Connected(Mutex<OwnedWriteHalf>),
}

pub struct TunnelSocket {
    pub(super) socket_id: Uuid,
    pub(super) write: WriteSocket,
    pub(super) sockets: TunnelSockets,
    pub(super) internal: Arc<InternalTunnelSocket>,
}

impl TunnelSocket {
    pub fn socket_id(&self) -> Uuid {
        self.socket_id
    }

    pub async fn shutdown(self) -> crate::Result<()> {
        if self.sockets.remove(&self.socket_id).is_some() {
            FriendsSocket::send_message(
                &self.write,
                ClientToServerMessage::SocketClose {
                    socket: self.socket_id,
                },
            )
            .await?;
            if let InternalTunnelSocket::Connected(ref stream) =
                *self.internal.clone()
            {
                stream.lock().await.shutdown().await?
            }
        }
        Ok(())
    }
}

impl Drop for TunnelSocket {
    fn drop(&mut self) {
        if self.sockets.remove(&self.socket_id).is_some() {
            let write = self.write.clone();
            let socket_id = self.socket_id;
            tokio::spawn(async move {
                let _ = FriendsSocket::send_message(
                    &write,
                    ClientToServerMessage::SocketClose { socket: socket_id },
                )
                .await;
            });
        }
    }
}
