use crate::auth::validate::get_user_record_from_bearer_token;
use crate::auth::AuthenticationError;
use crate::database::models::friend_item::FriendItem;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::models::users::User;
use crate::queue::session::AuthQueue;
use crate::queue::socket::{
    ActiveSocket, ActiveSockets, TunnelSocket, TunnelSocketType,
};
use crate::routes::ApiError;
use actix_web::web::{Data, Payload};
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_ws::Message;
use chrono::Utc;
use dashmap::mapref::entry::Entry;
use either::Either;
use futures_util::{StreamExt, TryStreamExt};
use rust_common::ids::UserId;
use rust_common::networking::message::{
    ClientToServerMessage, ServerToClientMessage,
};
use rust_common::users::UserStatus;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use Entry::Vacant;

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

    if let Some((_, socket)) = db.sockets.remove(&user.id) {
        let _ = socket.socket.close(None).await;
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
                db.sockets.get(
                    &if x.user_id == user.id.into() {
                        x.friend_id
                    } else {
                        x.user_id
                    }
                    .into(),
                )
            })
            .map(|x| x.status.clone())
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

    let db = db.clone();
    db.sockets
        .entry(user.id)
        .insert(ActiveSocket::new(status.clone(), session));

    broadcast_friends(
        user.id,
        ServerToClientMessage::StatusUpdate { status },
        &pool,
        &db,
        Some(friends),
    )
    .await?;

    let mut stream = msg_stream.into_stream();

    actix_web::rt::spawn(async move {
        let Some(active_socket) = db.sockets.get(&user.id) else {
            return;
        };
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            let message = match msg {
                Ok(Message::Text(text)) => {
                    ClientToServerMessage::deserialize(Either::Left(&text))
                }

                Ok(Message::Binary(bytes)) => {
                    ClientToServerMessage::deserialize(Either::Right(&bytes))
                }

                Ok(Message::Close(_)) => {
                    let _ = close_socket(user.id, &pool, &db).await;
                    continue;
                }

                Ok(Message::Ping(msg)) => {
                    if let Some(socket) = db.sockets.get(&user.id) {
                        let _ = socket.socket.clone().pong(&msg).await;
                    }
                    continue;
                }

                _ => continue,
            };

            if message.is_err() {
                continue;
            }

            match message.unwrap() {
                ClientToServerMessage::StatusUpdate { profile_name } => {
                    if let Some(mut pair) = db.sockets.get_mut(&user.id) {
                        let ActiveSocket { status, .. } = pair.value_mut();

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

                        let _ = broadcast_friends(
                            user.id,
                            ServerToClientMessage::StatusUpdate {
                                status: user_status,
                            },
                            &pool,
                            &db,
                            None,
                        )
                        .await;
                    }
                }

                ClientToServerMessage::SocketListen { socket } => {
                    let Vacant(entry) = db.tunnel_sockets.entry(socket) else {
                        continue;
                    };
                    entry.insert(TunnelSocket::new(
                        user.id,
                        TunnelSocketType::Listening,
                    ));
                    active_socket.owned_tunnel_sockets.insert(socket);
                    let _ = broadcast_friends(
                        user.id,
                        ServerToClientMessage::FriendSocketOpened {
                            user: user.id,
                            socket,
                        },
                        &pool,
                        &db,
                        None,
                    )
                    .await;
                }
                ClientToServerMessage::SocketConnect {
                    from_socket,
                    to_socket,
                } => {
                    let Vacant(entry) = db.tunnel_sockets.entry(from_socket)
                    else {
                        continue;
                    };
                    let Some(other_tunnel) = db.tunnel_sockets.get(&to_socket)
                    else {
                        continue;
                    };
                    let Some(other_user) = db.sockets.get(&other_tunnel.owner)
                    else {
                        continue;
                    };
                    let new_socket_id = Uuid::new_v4();
                    entry.insert(TunnelSocket::new(
                        user.id,
                        TunnelSocketType::Connected {
                            connected_to: new_socket_id,
                        },
                    ));
                    active_socket.owned_tunnel_sockets.insert(from_socket);
                    db.tunnel_sockets.insert(
                        new_socket_id,
                        TunnelSocket::new(
                            *other_user.key(),
                            TunnelSocketType::Connected {
                                connected_to: from_socket,
                            },
                        ),
                    );
                    other_user.owned_tunnel_sockets.insert(new_socket_id);
                    let _ = send_message(
                        &other_user,
                        &ServerToClientMessage::SocketConnected {
                            to_socket,
                            new_socket: new_socket_id,
                        },
                    )
                    .await;
                }
                ClientToServerMessage::SocketClose { socket } => {
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
                            let _ = broadcast_friends(
                                user.id,
                                ServerToClientMessage::SocketClosed { socket },
                                &pool,
                                &db,
                                None,
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
                    if tunnel_socket.owner != user.id {
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
    // FIXME Probably shouldn't be using database errors for this
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
            if let Some(socket) = sockets.sockets.get(&friend_id.into()) {
                let _ = send_message(socket.value(), &message).await;
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

pub async fn close_socket(
    id: UserId,
    pool: &PgPool,
    db: &ActiveSockets,
) -> Result<(), crate::database::models::DatabaseError> {
    if let Some((_, socket)) = db.sockets.remove(&id) {
        let _ = socket.socket.close(None).await;

        broadcast_friends(
            id,
            ServerToClientMessage::UserOffline { id },
            pool,
            db,
            None,
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
                    let _ = broadcast_friends(
                        id,
                        ServerToClientMessage::SocketClosed {
                            socket: owned_socket,
                        },
                        pool,
                        db,
                        None,
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
