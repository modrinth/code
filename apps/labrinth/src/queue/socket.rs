//! "Database" for Hydra
use crate::models::users::{UserId, UserStatus};
use actix_ws::Session;
use dashmap::DashMap;

pub struct ActiveSockets {
    pub sockets: DashMap<UserId, ActiveSocket>,
}

impl Default for ActiveSockets {
    fn default() -> Self {
        Self {
            sockets: DashMap::new(),
        }
    }
}

pub struct ActiveSocket {
    pub status: UserStatus,
    pub socket: Session,
}

impl ActiveSocket {
    pub fn new(status: UserStatus, session: Session) -> Self {
        Self {
            status,
            socket: session,
        }
    }
}
