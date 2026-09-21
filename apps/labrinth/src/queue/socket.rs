//! "Database" for Hydra

use crate::models::users::UserStatus;
use actix_ws::Session;
use ariadne::ids::UserId;
use dashmap::{DashMap, DashSet};
use prometheus::{IntGauge, Registry};
use std::sync::atomic::AtomicU32;
use std::time::Duration;
use uuid::Uuid;

pub type SocketId = u32;

pub struct ActiveSockets {
    pub sockets: DashMap<SocketId, ActiveSocket>,
    pub sockets_by_user_id: DashMap<UserId, DashSet<SocketId>>,
    pub next_socket_id: AtomicU32,
    pub tunnel_sockets: DashMap<Uuid, TunnelSocket>,
}

impl Default for ActiveSockets {
    fn default() -> Self {
        Self {
            sockets: DashMap::new(),
            sockets_by_user_id: DashMap::new(),
            next_socket_id: AtomicU32::new(0),
            tunnel_sockets: DashMap::new(),
        }
    }
}

impl ActiveSockets {
    pub fn get_status(&self, user: UserId) -> Option<UserStatus> {
        self.sockets_by_user_id
            .get(&user)
            .and_then(|x| x.iter().next().and_then(|x| self.sockets.get(&*x)))
            .map(|x| x.status.clone())
    }

    pub fn register_and_set_metrics(
        self: &std::sync::Arc<Self>,
        registry: &Registry,
    ) -> Result<(), prometheus::Error> {
        let active_sockets = IntGauge::new(
            "labrinth_active_sockets",
            "Number of currently connected launcher websockets",
        )?;
        registry.register(Box::new(active_sockets.clone()))?;

        let sockets = self.clone();
        tokio::spawn(async move {
            loop {
                active_sockets.set(sockets.sockets.len() as i64);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        Ok(())
    }
}

pub struct ActiveSocket {
    pub status: UserStatus,
    pub socket: Session,
    pub owned_tunnel_sockets: DashSet<Uuid>,
}

impl ActiveSocket {
    pub fn new(status: UserStatus, session: Session) -> Self {
        Self {
            status,
            socket: session,
            owned_tunnel_sockets: DashSet::new(),
        }
    }
}

pub struct TunnelSocket {
    pub owner: SocketId,
    pub socket_type: TunnelSocketType,
}

impl TunnelSocket {
    pub fn new(owner: SocketId, socket_type: TunnelSocketType) -> Self {
        Self { owner, socket_type }
    }
}

pub enum TunnelSocketType {
    Listening,
    Connected { connected_to: Uuid },
}
