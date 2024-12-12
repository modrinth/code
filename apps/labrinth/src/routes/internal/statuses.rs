use crate::auth::validate::get_user_record_from_bearer_token;
use crate::auth::AuthenticationError;
use crate::database::models::friend_item::FriendItem;
use crate::database::redis::RedisPool;
use crate::models::ids::UserId;
use crate::models::pats::Scopes;
use crate::models::users::{User, UserStatus};
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
    StatusUpdate { profile_name: Option<String> },
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerToClientMessage {
    StatusUpdate { status: UserStatus },
    UserOffline { id: UserId },
    FriendStatuses { statuses: Vec<UserStatus> },
    FriendRequest { from: UserId },
    FriendRequestRejected { from: UserId },
}

#[derive(Deserialize)]
struct LauncherHeartbeatInit {
    code: String,
}

#[get("launcher_socket")]
pub async fn ws_init(
    req: HttpRequest,
    pool: Data<PgPool>,
    web::Query(auth): web::Query<LauncherHeartbeatInit>,
    body: Payload,
    db: Data<ActiveSockets>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (scopes, db_user) = get_user_record_from_bearer_token(
        &req,
        Some(&auth.code),
        &**pool,
        &redis,
        &session_queue,
    )
    .await?
    .ok_or_else(|| {
        ApiError::Authentication(AuthenticationError::InvalidCredentials)
    })?;

    if !scopes.contains(Scopes::SESSION_ACCESS) {
        return Err(ApiError::Authentication(
            AuthenticationError::InvalidCredentials,
        ));
    }

    let user = User::from_full(db_user);

    if let Some((_, (_, session))) = db.auth_sockets.remove(&user.id) {
        let _ = session.close(None).await;
    }

    let (res, mut session, msg_stream) = match actix_ws::handle(&req, body) {
        Ok(x) => x,
        Err(e) => return Ok(e.error_response()),
    };

    let status = UserStatus {
        user_id: user.id,
        profile_name: None,
        last_update: Utc::now(),
    };

    let friends =
        FriendItem::get_user_friends(user.id.into(), Some(true), &**pool)
            .await?;

    let friend_statuses = if !friends.is_empty() {
        friends
            .iter()
            .filter_map(|x| {
                db.auth_sockets.get(
                    &if x.user_id == user.id.into() {
                        x.friend_id
                    } else {
                        x.user_id
                    }
                    .into(),
                )
            })
            .map(|x| x.value().0.clone())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let _ = session
        .text(serde_json::to_string(
            &ServerToClientMessage::FriendStatuses {
                statuses: friend_statuses,
            },
        )?)
        .await;

    db.auth_sockets.insert(user.id, (status.clone(), session));

    broadcast_friends(
        user.id,
        ServerToClientMessage::StatusUpdate { status },
        &pool,
        &db,
        Some(friends),
    )
    .await?;

    let mut stream = msg_stream.aggregate_continuations();

    actix_web::rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    if let Ok(message) =
                        serde_json::from_str::<ClientToServerMessage>(&text)
                    {
                        match message {
                            ClientToServerMessage::StatusUpdate {
                                profile_name,
                            } => {
                                if let Some(mut pair) =
                                    db.auth_sockets.get_mut(&user.id)
                                {
                                    let (status, _) = pair.value_mut();

                                    if status
                                        .profile_name
                                        .as_ref()
                                        .map(|x| x.len() > 64)
                                        .unwrap_or(false)
                                    {
                                        continue;
                                    }

                                    status.profile_name = profile_name;
                                    status.last_update = Utc::now();

                                    let _ = broadcast_friends(
                                        user.id,
                                        ServerToClientMessage::StatusUpdate {
                                            status: status.clone(),
                                        },
                                        &pool,
                                        &db,
                                        None,
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }

                Ok(AggregatedMessage::Close(_)) => {
                    let _ = close_socket(user.id, &pool, &db).await;
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    if let Some(mut socket) = db.auth_sockets.get_mut(&user.id)
                    {
                        let (_, socket) = socket.value_mut();
                        let _ = socket.pong(&msg).await;
                    }
                }

                _ => {}
            }
        }

        let _ = close_socket(user.id, &pool, &db).await;
    });

    Ok(res)
}

pub async fn broadcast_friends(
    user_id: UserId,
    message: ServerToClientMessage,
    pool: &PgPool,
    sockets: &ActiveSockets,
    friends: Option<Vec<FriendItem>>,
) -> Result<(), crate::database::models::DatabaseError> {
    let friends = if let Some(friends) = friends {
        friends
    } else {
        FriendItem::get_user_friends(user_id.into(), Some(true), pool).await?
    };

    for friend in friends {
        let friend_id = if friend.user_id == user_id.into() {
            friend.friend_id
        } else {
            friend.user_id
        };

        if friend.accepted {
            if let Some(mut socket) =
                sockets.auth_sockets.get_mut(&friend_id.into())
            {
                let (_, socket) = socket.value_mut();

                let _ = socket.text(serde_json::to_string(&message)?).await;
            }
        }
    }

    Ok(())
}

pub async fn close_socket(
    id: UserId,
    pool: &PgPool,
    sockets: &ActiveSockets,
) -> Result<(), crate::database::models::DatabaseError> {
    if let Some((_, (_, socket))) = sockets.auth_sockets.remove(&id) {
        let _ = socket.close(None).await;

        broadcast_friends(
            id,
            ServerToClientMessage::UserOffline { id },
            pool,
            sockets,
            None,
        )
        .await?;
    }

    Ok(())
}
