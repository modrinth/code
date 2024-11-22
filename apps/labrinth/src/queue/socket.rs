//! "Database" for Hydra
use crate::models::users::UserId;
use actix_ws::Session;
use dashmap::DashMap;

pub struct ActiveSockets {
    pub auth_sockets: DashMap<UserId, Session>,
}

impl Default for ActiveSockets {
    fn default() -> Self {
        Self {
            auth_sockets: DashMap::new(),
        }
    }
}
