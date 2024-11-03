//! "Database" for Hydra
use actix_ws::Session;
use dashmap::DashMap;

pub struct ActiveSockets {
    pub auth_sockets: DashMap<String, Session>,
}

impl Default for ActiveSockets {
    fn default() -> Self {
        Self {
            auth_sockets: DashMap::new(),
        }
    }
}
