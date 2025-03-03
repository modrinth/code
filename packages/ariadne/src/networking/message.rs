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

    SocketListen {
        socket: Uuid,
    },
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

    FriendSocketListening {
        user: UserId,
        socket: Uuid,
    },
    FriendSocketStoppedListening {
        user: UserId,
    },

    SocketConnected {
        to_socket: Uuid,
        new_socket: Uuid,
    },
    SocketClosed {
        socket: Uuid,
    },
    SocketData {
        socket: Uuid,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
}
