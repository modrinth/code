use crate::auth::get_user_from_headers;
use crate::database::models::DBUserId;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::models::users::UserFriend;
use crate::queue::session::AuthQueue;
use crate::queue::socket::ActiveSockets;
use crate::routes::ApiError;
use crate::routes::internal::statuses::{
    broadcast_friends_message, send_message_to_user,
};
use crate::sync::friends::RedisFriendsMessage;
use crate::sync::status::get_user_status;
use actix_web::{HttpRequest, HttpResponse, delete, get, post, web};
use ariadne::networking::message::ServerToClientMessage;
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
        crate::database::models::DBUser::get(&string, &**pool, &redis).await?;

    if let Some(friend) = friend {
        let mut transaction = pool.begin().await?;

        if let Some(friend) =
            crate::database::models::friend_item::DBFriend::get_friend(
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

            crate::database::models::friend_item::DBFriend::update_friend(
                friend.user_id,
                friend.friend_id,
                true,
                &mut transaction,
            )
            .await?;

            async fn send_friend_status(
                user_id: DBUserId,
                friend_id: DBUserId,
                sockets: &ActiveSockets,
                redis: &RedisPool,
            ) -> Result<(), ApiError> {
                if let Some(friend_status) =
                    get_user_status(user_id.into(), sockets, redis).await
                {
                    broadcast_friends_message(
                        redis,
                        RedisFriendsMessage::DirectStatusUpdate {
                            to_user: friend_id.into(),
                            status: friend_status,
                        },
                    )
                    .await?;
                }

                Ok(())
            }

            send_friend_status(friend.user_id, friend.friend_id, &db, &redis)
                .await?;
            send_friend_status(friend.friend_id, friend.user_id, &db, &redis)
                .await?;
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

            crate::database::models::friend_item::DBFriend {
                user_id: user.id.into(),
                friend_id: friend.id,
                created: Utc::now(),
                accepted: false,
            }
            .insert(&mut transaction)
            .await?;

            send_message_to_user(
                &db,
                friend.id.into(),
                &ServerToClientMessage::FriendRequest { from: user.id },
            )
            .await?;
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
        crate::database::models::DBUser::get(&string, &**pool, &redis).await?;

    if let Some(friend) = friend {
        let mut transaction = pool.begin().await?;

        crate::database::models::friend_item::DBFriend::remove(
            user.id.into(),
            friend.id,
            &mut transaction,
        )
        .await?;

        send_message_to_user(
            &db,
            friend.id.into(),
            &ServerToClientMessage::FriendRequestRejected { from: user.id },
        )
        .await?;

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
        crate::database::models::friend_item::DBFriend::get_user_friends(
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
