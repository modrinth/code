use crate::auth::get_user_from_headers;
use crate::database::models::UserId;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::models::users::UserFriend;
use crate::queue::session::AuthQueue;
use crate::queue::socket::ActiveSockets;
use crate::routes::internal::statuses::{close_socket, ServerToClientMessage};
use crate::routes::ApiError;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(add_friend);
    cfg.service(remove_friend);
    cfg.service(friends);
}

#[post("friend/{id}")]
pub async fn add_friend(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    db: web::Data<ActiveSockets>,
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

    let string = info.into_inner().0;
    let friend =
        crate::database::models::User::get(&string, &**pool, &redis).await?;

    if let Some(friend) = friend {
        let mut transaction = pool.begin().await?;

        if let Some(friend) =
            crate::database::models::friend_item::FriendItem::get_friend(
                user.id.into(),
                friend.id,
                &**pool,
            )
            .await?
        {
            if friend.accepted {
                return Err(ApiError::InvalidInput(
                    "You are already friends with this user!".to_string(),
                ));
            }

            if !friend.accepted && user.id != friend.friend_id.into() {
                return Err(ApiError::InvalidInput(
                    "You cannot accept your own friend request!".to_string(),
                ));
            }

            crate::database::models::friend_item::FriendItem::update_friend(
                friend.user_id,
                friend.friend_id,
                true,
                &mut transaction,
            )
            .await?;

            async fn send_friend_status(
                user_id: UserId,
                friend_id: UserId,
                sockets: &ActiveSockets,
            ) -> Result<(), ApiError> {
                if let Some(pair) = sockets.auth_sockets.get(&user_id.into()) {
                    let (friend_status, _) = pair.value();
                    if let Some(socket) =
                        sockets.auth_sockets.get(&friend_id.into())
                    {
                        let (_, socket) = socket.value();

                        let _ = socket
                            .clone()
                            .text(serde_json::to_string(
                                &ServerToClientMessage::StatusUpdate {
                                    status: friend_status.clone(),
                                },
                            )?)
                            .await;
                    }
                }

                Ok(())
            }

            send_friend_status(friend.user_id, friend.friend_id, &db).await?;
            send_friend_status(friend.friend_id, friend.user_id, &db).await?;
        } else {
            if friend.id == user.id.into() {
                return Err(ApiError::InvalidInput(
                    "You cannot add yourself as a friend!".to_string(),
                ));
            }

            if !friend.allow_friend_requests {
                return Err(ApiError::InvalidInput(
                    "Friend requests are disabled for this user!".to_string(),
                ));
            }

            crate::database::models::friend_item::FriendItem {
                user_id: user.id.into(),
                friend_id: friend.id,
                created: Utc::now(),
                accepted: false,
            }
            .insert(&mut transaction)
            .await?;

            if let Some(socket) = db.auth_sockets.get(&friend.id.into()) {
                let (_, socket) = socket.value();

                if socket
                    .clone()
                    .text(serde_json::to_string(
                        &ServerToClientMessage::FriendRequest { from: user.id },
                    )?)
                    .await
                    .is_err()
                {
                    close_socket(user.id, &pool, &db).await?;
                }
            }
        }

        transaction.commit().await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound)
    }
}

#[delete("friend/{id}")]
pub async fn remove_friend(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    db: web::Data<ActiveSockets>,
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

    let string = info.into_inner().0;
    let friend =
        crate::database::models::User::get(&string, &**pool, &redis).await?;

    if let Some(friend) = friend {
        let mut transaction = pool.begin().await?;

        crate::database::models::friend_item::FriendItem::remove(
            user.id.into(),
            friend.id,
            &mut transaction,
        )
        .await?;

        if let Some(socket) = db.auth_sockets.get(&friend.id.into()) {
            let (_, socket) = socket.value();

            let _ = socket
                .clone()
                .text(serde_json::to_string(
                    &ServerToClientMessage::FriendRequestRejected {
                        from: user.id,
                    },
                )?)
                .await;
        }

        transaction.commit().await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound)
    }
}

#[get("friends")]
pub async fn friends(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::USER_READ]),
    )
    .await?
    .1;

    let friends =
        crate::database::models::friend_item::FriendItem::get_user_friends(
            user.id.into(),
            None,
            &**pool,
        )
        .await?
        .into_iter()
        .map(UserFriend::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(friends))
}
