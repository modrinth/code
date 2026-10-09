use super::{DiscordGuard, FriendsSocket};

/// Long-running social integrations.
pub struct Presence {
    pub discord_rpc: DiscordGuard,
    pub friends_socket: FriendsSocket,
}

impl Presence {
    pub(crate) fn init() -> crate::Result<Self> {
        Ok(Self {
            discord_rpc: DiscordGuard::init()?,
            friends_socket: FriendsSocket::new(),
        })
    }
}
