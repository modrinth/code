use crate::auth::AuthenticationError;
use crate::auth::validate::get_user_record_from_bearer_token;
use crate::database::models::friend_item::DBFriend;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::models::users::User;
use crate::queue::session::AuthQueue;
use crate::queue::socket::{
    ActiveSocket, ActiveSockets, SocketId, TunnelSocketType,
};
use crate::routes::ApiError;
use crate::sync::friends::{FRIENDS_CHANNEL_NAME, RedisFriendsMessage};
use crate::sync::status::{
    get_user_status, push_back_user_expiry, replace_user_status,
};
use actix_web::web::{Data, Payload};
use actix_web::{HttpRequest, HttpResponse, get, web};
use actix_ws::Message;
use ariadne::ids::UserId;
use ariadne::networking::message::{
    ClientToServerMessage, ServerToClientMessage,
};
use ariadne::users::UserStatus;
use chrono::Utc;
use either::Either;
use futures_util::future::select;
use futures_util::{StreamExt, TryStreamExt};
use redis::AsyncCommands;
use serde::Deserialize;
use sqlx::PgPool;
use std::pin::pin;
use std::sync::atomic::Ordering;
use tokio::sync::oneshot::error::TryRecvError;
use tokio::time::{Duration, sleep};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(ws_init);
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
    let user_id = user.id;

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
        DBFriend::get_user_friends(user.id.into(), Some(true), &**pool).await?;

    let friend_statuses = if !friends.is_empty() {
        let db = db.clone();
        let redis = redis.clone();
        tokio_stream::iter(friends.iter())
            .map(|x| {
                let db = db.clone();
                let redis = redis.clone();
                async move {
                    get_user_status(
                        if x.user_id == user_id.into() {
                            x.friend_id
                        } else {
                            x.user_id
                        }
                        .into(),
                        &db,
                        &redis,
                    )
                    .await
                }
            })
            .buffer_unordered(16)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect()
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

    let db = db.clone();
    let socket_id = db.next_socket_id.fetch_add(1, Ordering::Relaxed);
    db.sockets
        .insert(socket_id, ActiveSocket::new(status.clone(), session));
    db.sockets_by_user_id
        .entry(user.id)
        .or_default()
        .insert(socket_id);

    #[cfg(debug_assertions)]
    tracing::info!("Connection {socket_id} opened by {}", user.id);

    replace_user_status(None, Some(&status), &redis).await?;
    broadcast_friends_message(
        &redis,
        RedisFriendsMessage::StatusUpdate { status },
    )
    .await?;

    let (shutdown_sender, mut shutdown_receiver) =
        tokio::sync::oneshot::channel::<()>();

    {
        let db = db.clone();
        let redis = redis.clone();
        actix_web::rt::spawn(async move {
            while shutdown_receiver.try_recv() == Err(TryRecvError::Empty) {
                sleep(Duration::from_secs(30)).await;
                if let Some(socket) = db.sockets.get(&socket_id) {
                    let _ = socket.socket.clone().ping(&[]).await;
                }
                let _ = push_back_user_expiry(user_id, &redis).await;
            }
        });
    }

    let mut stream = msg_stream.into_stream();

    actix_web::rt::spawn(async move {
        loop {
            let next = pin!(stream.next());
            let timeout = pin!(sleep(Duration::from_secs(30)));
            let futures_util::future::Either::Left((Some(msg), _)) =
                select(next, timeout).await
            else {
                break;
            };

            let message = match msg {
                Ok(Message::Text(text)) => {
                    ClientToServerMessage::deserialize(Either::Left(&text))
                }

                Ok(Message::Binary(bytes)) => {
                    ClientToServerMessage::deserialize(Either::Right(&bytes))
                }

                Ok(Message::Close(_)) => break,

                Ok(Message::Ping(msg)) => {
                    if let Some(socket) = db.sockets.get(&socket_id) {
                        let _ = socket.socket.clone().pong(&msg).await;
                    }
                    continue;
                }

                _ => continue,
            };

            if message.is_err() {
                continue;
            }
            let message = message.unwrap();

            #[cfg(debug_assertions)]
            if !message.is_binary() {
                tracing::info!(
                    "Received message from {socket_id}: {message:?}"
                );
            }

            match message {
                ClientToServerMessage::StatusUpdate { profile_name } => {
                    if let Some(mut pair) = db.sockets.get_mut(&socket_id) {
                        let ActiveSocket { status, .. } = pair.value_mut();

                        let old_status = status.clone();

                        if status
                            .profile_name
                            .as_ref()
                            .map(|x| x.len() > 64)
                            .unwrap_or(false)
                        {
                            return;
                        }

                        status.profile_name = profile_name;
                        status.last_update = Utc::now();

                        let user_status = status.clone();
                        // We drop the pair to avoid holding the lock for too long
                        drop(pair);

                        let _ = replace_user_status(
                            Some(&old_status),
                            Some(&user_status),
                            &redis,
                        )
                        .await;
                        let _ = broadcast_friends_message(
                            &redis,
                            RedisFriendsMessage::StatusUpdate {
                                status: user_status,
                            },
                        )
                        .await;
                    }
                }

                ClientToServerMessage::SocketListen { .. } => {
                    // TODO: Listen to socket
                    // The code below probably won't need changes, but there's no way to connect to
                    // a tunnel socket yet, so we shouldn't be storing them

                    // let Some(active_socket) = db.sockets.get(&socket_id) else {
                    //     return;
                    // };
                    // let Vacant(entry) = db.tunnel_sockets.entry(socket) else {
                    //     continue;
                    // };
                    // entry.insert(TunnelSocket::new(
                    //     socket_id,
                    //     TunnelSocketType::Listening,
                    // ));
                    // active_socket.owned_tunnel_sockets.insert(socket);
                    // let _ = broadcast_friends(
                    //     user.id,
                    //     ServerToClientMessage::FriendSocketListening {
                    //         user: user.id,
                    //         socket,
                    //     },
                    //     &pool,
                    //     &db,
                    //     None,
                    // )
                    // .await;
                }
                ClientToServerMessage::SocketClose { socket } => {
                    let Some(active_socket) = db.sockets.get(&socket_id) else {
                        return;
                    };
                    if active_socket
                        .owned_tunnel_sockets
                        .remove(&socket)
                        .is_none()
                    {
                        continue;
                    }
                    let Some((_, tunnel_socket)) =
                        db.tunnel_sockets.remove(&socket)
                    else {
                        continue;
                    };
                    match tunnel_socket.socket_type {
                        TunnelSocketType::Listening => {
                            let _ = broadcast_to_local_friends(
                                user.id,
                                ServerToClientMessage::FriendSocketStoppedListening { user: user.id },
                                &pool,
                                &db,
                            )
                            .await;
                        }
                        TunnelSocketType::Connected { connected_to } => {
                            let Some((_, other)) =
                                db.tunnel_sockets.remove(&connected_to)
                            else {
                                continue;
                            };
                            let Some(other_user) = db.sockets.get(&other.owner)
                            else {
                                continue;
                            };
                            let _ = send_message(
                                &other_user,
                                &ServerToClientMessage::SocketClosed { socket },
                            )
                            .await;
                        }
                    }
                }
                ClientToServerMessage::SocketSend { socket, data } => {
                    let Some(tunnel_socket) = db.tunnel_sockets.get(&socket)
                    else {
                        continue;
                    };
                    if tunnel_socket.owner != socket_id {
                        continue;
                    }
                    let TunnelSocketType::Connected { connected_to } =
                        tunnel_socket.socket_type
                    else {
                        continue;
                    };
                    let Some(other_tunnel) =
                        db.tunnel_sockets.get(&connected_to)
                    else {
                        continue;
                    };
                    let Some(other_user) = db.sockets.get(&other_tunnel.owner)
                    else {
                        continue;
                    };
                    let _ = send_message(
                        &other_user,
                        &ServerToClientMessage::SocketData {
                            socket: connected_to,
                            data,
                        },
                    )
                    .await;
                }
            }
        }

        let _ = shutdown_sender.send(());
        let _ = close_socket(socket_id, &pool, &db, &redis).await;
    });

    Ok(res)
}

pub async fn broadcast_friends_message(
    redis: &RedisPool,
    message: RedisFriendsMessage,
) -> Result<(), crate::database::models::DatabaseError> {
    let _: () = redis
        .pool
        .get()
        .await?
        .publish(FRIENDS_CHANNEL_NAME, message)
        .await?;
    Ok(())
}

pub async fn broadcast_to_local_friends(
    user_id: UserId,
    message: ServerToClientMessage,
    pool: &PgPool,
    sockets: &ActiveSockets,
) -> Result<(), crate::database::models::DatabaseError> {
    broadcast_to_known_local_friends(
        user_id,
        message,
        sockets,
        DBFriend::get_user_friends(user_id.into(), Some(true), pool).await?,
    )
    .await
}

async fn broadcast_to_known_local_friends(
    user_id: UserId,
    message: ServerToClientMessage,
    sockets: &ActiveSockets,
    friends: Vec<DBFriend>,
) -> Result<(), crate::database::models::DatabaseError> {
    // FIXME Probably shouldn't be using database errors for this. Maybe ApiError?

    for friend in friends {
        let friend_id = if friend.user_id == user_id.into() {
            friend.friend_id
        } else {
            friend.user_id
        };

        if friend.accepted {
            if let Some(socket_ids) =
                sockets.sockets_by_user_id.get(&friend_id.into())
            {
                for socket_id in socket_ids.iter() {
                    if let Some(socket) = sockets.sockets.get(&socket_id) {
                        let _ = send_message(socket.value(), &message).await;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn send_message(
    socket: &ActiveSocket,
    message: &ServerToClientMessage,
) -> Result<(), crate::database::models::DatabaseError> {
    let mut socket = socket.socket.clone();

    // FIXME Probably shouldn't swallow sending errors
    let _ = match message.serialize() {
        Ok(Either::Left(text)) => socket.text(text).await,
        Ok(Either::Right(bytes)) => socket.binary(bytes).await,
        Err(_) => Ok(()), // TODO: Maybe should log these? Though it is the backend
    };

    Ok(())
}

pub async fn send_message_to_user(
    db: &ActiveSockets,
    user: UserId,
    message: &ServerToClientMessage,
) -> Result<(), crate::database::models::DatabaseError> {
    if let Some(socket_ids) = db.sockets_by_user_id.get(&user) {
        for socket_id in socket_ids.iter() {
            if let Some(socket) = db.sockets.get(&socket_id) {
                send_message(&socket, message).await?;
            }
        }
    }

    Ok(())
}

pub async fn close_socket(
    id: SocketId,
    pool: &PgPool,
    db: &ActiveSockets,
    redis: &RedisPool,
) -> Result<(), crate::database::models::DatabaseError> {
    if let Some((_, socket)) = db.sockets.remove(&id) {
        let user_id = socket.status.user_id;
        db.sockets_by_user_id.remove_if(&user_id, |_, sockets| {
            sockets.remove(&id);
            sockets.is_empty()
        });

        let _ = socket.socket.close(None).await;

        replace_user_status(Some(&socket.status), None, redis).await?;
        broadcast_friends_message(
            redis,
            RedisFriendsMessage::UserOffline { user: user_id },
        )
        .await?;

        for owned_socket in socket.owned_tunnel_sockets {
            let Some((_, tunnel_socket)) =
                db.tunnel_sockets.remove(&owned_socket)
            else {
                continue;
            };
            match tunnel_socket.socket_type {
                TunnelSocketType::Listening => {
                    let _ = broadcast_to_local_friends(
                        user_id,
                        ServerToClientMessage::SocketClosed {
                            socket: owned_socket,
                        },
                        pool,
                        db,
                    )
                    .await;
                }
                TunnelSocketType::Connected { connected_to } => {
                    let Some((_, other)) =
                        db.tunnel_sockets.remove(&connected_to)
                    else {
                        continue;
                    };
                    let Some(other_user) = db.sockets.get(&other.owner) else {
                        continue;
                    };
                    let _ = send_message(
                        &other_user,
                        &ServerToClientMessage::SocketClosed {
                            socket: connected_to,
                        },
                    )
                    .await;
                }
            }
        }
    }

    Ok(())
}
