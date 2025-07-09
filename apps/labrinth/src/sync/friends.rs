use crate::queue::socket::ActiveSockets;
use crate::routes::internal::statuses::{
    broadcast_to_local_friends, send_message_to_user,
};
use actix_web::web::Data;
use ariadne::ids::UserId;
use ariadne::networking::message::ServerToClientMessage;
use ariadne::users::UserStatus;
use redis::aio::PubSub;
use redis::{RedisWrite, ToRedisArgs};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio_stream::StreamExt;

pub const FRIENDS_CHANNEL_NAME: &str = "friends";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RedisFriendsMessage {
    StatusUpdate { status: UserStatus },
    UserOffline { user: UserId },
    DirectStatusUpdate { to_user: UserId, status: UserStatus },
}

impl ToRedisArgs for RedisFriendsMessage {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        out.write_arg(&serde_json::to_vec(&self).unwrap())
    }
}

pub async fn handle_pubsub(
    mut pubsub: PubSub,
    pool: PgPool,
    sockets: Data<ActiveSockets>,
) {
    pubsub.subscribe(FRIENDS_CHANNEL_NAME).await.unwrap();
    let mut stream = pubsub.into_on_message();
    while let Some(message) = stream.next().await {
        if message.get_channel_name() != FRIENDS_CHANNEL_NAME {
            continue;
        }
        let payload = serde_json::from_slice(message.get_payload_bytes());

        let pool = pool.clone();
        let sockets = sockets.clone();
        actix_rt::spawn(async move {
            match payload {
                Ok(RedisFriendsMessage::StatusUpdate { status }) => {
                    let _ = broadcast_to_local_friends(
                        status.user_id,
                        ServerToClientMessage::StatusUpdate { status },
                        &pool,
                        &sockets,
                    )
                    .await;
                }

                Ok(RedisFriendsMessage::UserOffline { user }) => {
                    let _ = broadcast_to_local_friends(
                        user,
                        ServerToClientMessage::UserOffline { id: user },
                        &pool,
                        &sockets,
                    )
                    .await;
                }

                Ok(RedisFriendsMessage::DirectStatusUpdate {
                    to_user,
                    status,
                }) => {
                    let _ = send_message_to_user(
                        &sockets,
                        to_user,
                        &ServerToClientMessage::StatusUpdate { status },
                    )
                    .await;
                }

                Err(_) => {}
            }
        });
    }
}
