use crate::ids::UserId;
use crate::users::UserStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientToServerMessage {
    StatusUpdate {
        profile_name: Option<String>,
    },

    SocketOpen,
    SocketClose {
        socket: Uuid,
    },
    SocketSend {
        socket: Uuid,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerToClientMessage {
    StatusUpdate {
        status: UserStatus,
    },
    UserOffline {
        id: UserId,
    },
    FriendStatuses {
        statuses: Vec<UserStatus>,
    },
    FriendRequest {
        from: UserId,
    },
    FriendRequestRejected {
        from: UserId,
    },

    SocketOpened {
        socket: Uuid,
    },
    SocketClosed {
        socket: Uuid,
    },
    FriendSocketOpened {
        user: UserId,
        socket: Uuid,
    },
    SocketData {
        socket: Uuid,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
}
