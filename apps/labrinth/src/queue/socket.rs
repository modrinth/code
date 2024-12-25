//! "Database" for Hydra
use crate::models::users::{UserId, UserStatus};
use dashmap::DashMap;

pub struct ActiveSockets {
    pub auth_sockets: DashMap<UserId, (UserStatus)>,
}

impl Default for ActiveSockets {
    fn default() -> Self {
        Self {
            auth_sockets: DashMap::new(),
        }
    }
}
