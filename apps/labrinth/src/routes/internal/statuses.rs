use crate::auth::get_user_from_headers;
use crate::database::models::friend_item::FriendItem;
use crate::database::models::user_status_item::UserStatusItem;
use crate::database::redis::RedisPool;
use crate::models::ids::UserId;
use crate::models::pats::Scopes;
use crate::models::users::UserStatus;
use crate::queue::session::AuthQueue;
use crate::queue::socket::ActiveSockets;
use crate::routes::ApiError;
use actix_web::web::{Data, Payload};
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use chrono::Utc;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(ws_init);
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientToServerMessage {
    StatusUpdate(UserStatus),
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerToClientMessage {
    StatusUpdate(UserStatus),
    UserOffline(UserId),
    FriendStatuses(Vec<UserStatus>),
    FriendRequest(UserId),
}

#[get("launcher_heartbeat")]
pub async fn ws_init(
    req: HttpRequest,
    pool: Data<PgPool>,
    body: Payload,
    db: Data<ActiveSockets>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::USER_WRITE]),
    )
    .await?
    .1;

    if let Some((_, session)) = db.auth_sockets.remove(&user.id) {
        let _ = session.close(None).await;
    }

    let (res, mut session, msg_stream) = match actix_ws::handle(&req, body) {
        Ok(x) => x,
        Err(e) => return Ok(e.error_response()),
    };

    let status = UserStatusItem {
        id: user.id.into(),
        profile_name: None,
        last_update: Utc::now(),
    };
    status.set(&redis).await?;

    let friends = FriendItem::get_user_friends(user.id.into(), &**pool).await?;
    let friend_statuses = UserStatusItem::get_many(
        &friends.iter().map(|x| x.friend_id).collect::<Vec<_>>(),
        &redis,
    )
    .await?;
    let _ = session
        .text(serde_json::to_string(
            &ServerToClientMessage::FriendStatuses(
                friend_statuses.into_iter().map(|x| x.into()).collect(),
            ),
        )?)
        .await;
    db.auth_sockets.insert(user.id, session);

    broadcast_friends(
        user.id,
        ServerToClientMessage::StatusUpdate(status.into()),
        &pool,
        &redis,
        &db,
        Some(friends),
    )
    .await?;

    let mut stream = msg_stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    actix_web::rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    if let Ok(message) =
                        serde_json::from_str::<ClientToServerMessage>(&text)
                    {
                        match message {
                            ClientToServerMessage::StatusUpdate(status) => {
                                if status.user_id != user.id {
                                    continue;
                                }

                                let status_item: UserStatusItem =
                                    UserStatusItem {
                                        id: user.id.into(),
                                        profile_name: status
                                            .profile_name
                                            .clone(),
                                        last_update: Utc::now(),
                                    };
                                let _ = status_item.set(&redis).await;

                                let _ = broadcast_friends(
                                    user.id,
                                    ServerToClientMessage::StatusUpdate(status),
                                    &pool,
                                    &redis,
                                    &db,
                                    None,
                                )
                                .await;
                            }
                        }
                    }
                }

                Ok(AggregatedMessage::Close(_)) => {
                    let _ = close_socket(user.id, &pool, &redis, &db).await;
                }

                _ => {}
            }
        }
    });

    Ok(res)
}

pub async fn broadcast_friends(
    user_id: UserId,
    message: ServerToClientMessage,
    pool: &PgPool,
    redis: &RedisPool,
    sockets: &ActiveSockets,
    friends: Option<Vec<FriendItem>>,
) -> Result<(), crate::database::models::DatabaseError> {
    let friends = if let Some(friends) = friends {
        friends
    } else {
        FriendItem::get_user_friends(user_id.into(), pool).await?
    };

    if !friends.is_empty() {
        for friend in friends {
            if friend.accepted {
                if let Some(mut socket) =
                    sockets.auth_sockets.get_mut(&friend.friend_id.into())
                {
                    let socket = socket.value_mut();

                    // TODO: bulk close sockets for better perf
                    if socket
                        .text(serde_json::to_string(&message)?)
                        .await
                        .is_err()
                    {
                        Box::pin(close_socket(
                            friend.friend_id.into(),
                            pool,
                            redis,
                            sockets,
                        ))
                        .await?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn close_socket(
    id: UserId,
    pool: &PgPool,
    redis: &RedisPool,
    sockets: &ActiveSockets,
) -> Result<(), crate::database::models::DatabaseError> {
    if let Some((_, socket)) = sockets.auth_sockets.remove(&id) {
        let _ = socket.close(None).await;
    }

    UserStatusItem::remove(id.into(), redis).await?;
    broadcast_friends(
        id,
        ServerToClientMessage::UserOffline(id),
        pool,
        redis,
        sockets,
        None,
    )
    .await?;

    Ok(())
}
