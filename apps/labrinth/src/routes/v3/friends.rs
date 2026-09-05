use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::blocked_user_item::DBBlockedUser;
use crate::database::models::friend_item::DBFriend;
use crate::database::models::{DBUser, DBUserId};
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
use crate::util::error::ApiContext as _;
use crate::util::error::Context as _;
use actix_web::{HttpRequest, HttpResponse, delete, get, post, web};
use ariadne::networking::message::ServerToClientMessage;
use chrono::Utc;
use xredis::RedisPool;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(add_friend);
    cfg.service(remove_friend);
    cfg.service(friends);
}

/// Add a friend.  
#[utoipa::path(tag = "friends", responses((status = NO_CONTENT)))]
#[post("/friend/{id}")]
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
        Scopes::USER_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let string = info.into_inner().0;
    let Some(friend) = DBUser::get(&string, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
    else {
        return Err(ApiError::NotFound(eyre::eyre!("resource not found")));
    };

    if DBBlockedUser::is_blocked(friend.id, user.id.into(), &**pool)
        .await
        .wrap_internal_err("checking whether target user blocked requester")?
    {
        return Err(ApiError::Request(eyre::eyre!(
            "You've been blocked the other user!",
        )));
    } else if DBBlockedUser::is_blocked(user.id.into(), friend.id, &**pool)
        .await
        .wrap_internal_err("checking whether requester blocked target user")?
    {
        return Err(ApiError::Request(eyre::eyre!(
            "You've blocked the other user!",
        )));
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    if let Some(friend) =
        DBFriend::get_friend(user.id.into(), friend.id, &**pool)
            .await
            .wrap_internal_err("fetching friend from database")?
    {
        if friend.accepted {
            return Err(ApiError::Request(eyre::eyre!(
                "You are already friends with this user!",
            )));
        }

        if !friend.accepted && user.id != friend.friend_id.into() {
            return Err(ApiError::Request(eyre::eyre!(
                "You cannot accept your own friend request!",
            )));
        }

        DBFriend::update_friend(
            friend.user_id,
            friend.friend_id,
            true,
            &mut transaction,
        )
        .await
        .wrap_internal_err("updating friend in database")?;

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
                .await
                .wrap_internal_err("updating friend in database")?;
            }

            Ok(())
        }

        send_friend_status(friend.user_id, friend.friend_id, &db, &redis)
            .await
            .wrap_api_err("executing `send_friend_status`")?;
        send_friend_status(friend.friend_id, friend.user_id, &db, &redis)
            .await
            .wrap_api_err("executing `send_friend_status`")?;
    } else {
        if friend.id == user.id.into() {
            return Err(ApiError::Request(eyre::eyre!(
                "You cannot add yourself as a friend!",
            )));
        }

        if !friend.allow_friend_requests {
            return Err(ApiError::Request(eyre::eyre!(
                "Friend requests are disabled for this user!",
            )));
        }

        DBFriend {
            user_id: user.id.into(),
            friend_id: friend.id,
            created: Utc::now(),
            accepted: false,
        }
        .insert(&mut transaction)
        .await
        .wrap_internal_err(
            "inserting database records for `send_friend_status`",
        )?;

        send_message_to_user(
            &db,
            friend.id.into(),
            &ServerToClientMessage::FriendRequest { from: user.id },
        )
        .await
        .wrap_internal_err(
            "inserting database records for `send_friend_status`",
        )?;
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    Ok(HttpResponse::NoContent().body(""))
}

/// Remove a friend.  
#[utoipa::path(tag = "friends", responses((status = NO_CONTENT)))]
#[delete("/friend/{id}")]
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
        Scopes::USER_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let string = info.into_inner().0;
    let friend = DBUser::get(&string, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(friend) = friend {
        let mut transaction = pool
            .begin()
            .await
            .wrap_internal_err("starting database transaction")?;

        DBFriend::remove(user.id.into(), friend.id, &mut transaction)
            .await
            .wrap_internal_err("deleting friend from database")?;

        send_message_to_user(
            &db,
            friend.id.into(),
            &ServerToClientMessage::FriendRequestRejected { from: user.id },
        )
        .await
        .wrap_internal_err("deleting friend from database")?;

        transaction
            .commit()
            .await
            .wrap_internal_err("committing database transaction")?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

/// List friends.  
#[utoipa::path(tag = "friends", responses((status = OK, body = Vec<UserFriend>)))]
#[get("/friends")]
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
        Scopes::USER_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let friends = DBFriend::get_user_friends(user.id.into(), None, &**pool)
        .await
        .wrap_internal_err("fetching friends from database")?
        .into_iter()
        .map(UserFriend::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(friends))
}
