use crate::database::PgPool;
use crate::database::models::notification_item::DBNotification;
use crate::models::ids::NotificationId;
use crate::models::notifications::Notification;
use crate::queue::socket::ActiveSockets;
use crate::routes::internal::statuses::{
    broadcast_to_local_friends, send_message_to_user, send_notification_to_user,
};
use actix_web::web::Data;
use ariadne::ids::UserId;
use ariadne::networking::message::ServerToClientMessage;
use ariadne::users::UserStatus;
use redis::{RedisWrite, ToRedisArgs, ToSingleRedisArg};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

pub const FRIENDS_CHANNEL_NAME: &str = "friends:v1";

#[derive(Serialize, Deserialize)]
pub enum RedisFriendsMessage {
    StatusUpdate {
        status: UserStatus,
    },
    UserOffline {
        user: UserId,
    },
    DirectStatusUpdate {
        to_user: UserId,
        status: UserStatus,
    },
    Notification {
        to_user: UserId,
        notification_id: NotificationId,
    },
}

impl ToRedisArgs for RedisFriendsMessage {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        out.write_arg(&postcard::to_allocvec(&self).unwrap())
    }
}

impl ToSingleRedisArg for RedisFriendsMessage {}

pub async fn handle_pubsub(
    mut messages: mpsc::Receiver<Vec<u8>>,
    pool: PgPool,
    sockets: Data<ActiveSockets>,
) {
    while let Some(message) = messages.recv().await {
        let payload = postcard::from_bytes::<RedisFriendsMessage>(&message);

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

                Ok(RedisFriendsMessage::Notification {
                    to_user,
                    notification_id,
                }) => {
                    if let Ok(Some(notification)) =
                        DBNotification::get(notification_id.into(), &pool).await
                    {
                        let _ = send_notification_to_user(
                            &sockets,
                            to_user,
                            &Notification::from(notification),
                        )
                        .await;
                    }
                }

                Err(_) => {}
            }
        });
    }
}
