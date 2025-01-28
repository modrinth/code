//! "Database" for Hydra

use crate::models::users::{UserId, UserStatus};
use actix_ws::Session;
use dashmap::{DashMap, DashSet};
use uuid::Uuid;

pub struct ActiveSockets {
    pub sockets: DashMap<UserId, ActiveSocket>,
    pub tunnel_sockets: DashMap<Uuid, TunnelSocket>,
}

impl Default for ActiveSockets {
    fn default() -> Self {
        Self {
            sockets: DashMap::new(),
            tunnel_sockets: DashMap::new(),
        }
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
    pub owner: UserId,
    pub socket_type: TunnelSocketType,
}

impl TunnelSocket {
    pub fn new(owner: UserId, socket_type: TunnelSocketType) -> Self {
        Self { owner, socket_type }
    }
}

pub enum TunnelSocketType {
    Listening,
    Connected { connected_to: Uuid },
}
