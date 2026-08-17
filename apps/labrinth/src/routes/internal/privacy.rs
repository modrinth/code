use crate::database::PgPool;
use crate::database::models::blocked_user_item::DBBlockedUser;
use crate::database::models::friend_item::DBFriend;
use crate::database::models::user_item::DBUser;
use crate::database::models::user_preferences_item::DBUserPreferences;
use crate::models::v3::preferences::{InvitePrivacy, UserPreferences};
use crate::routes::ApiError;
use crate::util::error::Context as _;
use crate::util::guards::admin_key_guard;
use actix_web::{get, web};
use serde::Serialize;
use xredis::RedisPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(invite_privacy_status);
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct InvitePrivacyStatus {
    pub allowed: bool,
}

#[derive(strum::EnumString)]
#[strum(serialize_all = "snake_case")]
enum InviteType {
    SharedInstances,
    HostingAccess,
}

/// Check whether `user_id` may send `target_id` an invite of the given type.
#[utoipa::path(tag = "privacy", responses((status = OK, body = InvitePrivacyStatus)))]
#[get(
    "/privacy/{invite_type}/{user_id}/{target_id}",
    guard = "admin_key_guard"
)]
pub async fn invite_privacy_status(
    info: web::Path<(String, String, String)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<web::Json<InvitePrivacyStatus>, ApiError> {
    let (invite_type, user_id, target_id) = info.into_inner();

    let invite_type = invite_type
        .parse::<InviteType>()
        .wrap_request_err("invalid invite_type")?;
    let (user, target) = futures::try_join!(
        DBUser::get(&user_id, &**pool, &redis),
        DBUser::get(&target_id, &**pool, &redis),
    )
    .wrap_internal_err("resolving user ids")?;

    let user_id = user.wrap_not_found_err("user not found")?.id;
    let target_id = target.wrap_not_found_err("target not found")?.id;

    let blocked = DBBlockedUser::is_blocked(target_id, user_id, &**pool)
        .await
        .wrap_internal_err("checking whether user is blocked")?;

    let allowed = if blocked {
        false
    } else {
        let overrides = DBUserPreferences::get(target_id, &**pool)
            .await
            .wrap_internal_err("fetching user preferences")?;
        let preferences = UserPreferences::resolve(overrides);

        let privacy = match invite_type {
            InviteType::SharedInstances => {
                preferences.social.shared_instances_privacy
            }
            InviteType::HostingAccess => {
                preferences.social.hosting_access_privacy
            }
        };

        match privacy {
            InvitePrivacy::Everyone => true,
            InvitePrivacy::None => false,
            InvitePrivacy::Friends => {
                DBFriend::get_friend(user_id, target_id, &**pool)
                    .await
                    .wrap_internal_err("checking friendship status")?
                    .is_some_and(|friend| friend.accepted)
            }
        }
    };

    Ok(web::Json(InvitePrivacyStatus { allowed }))
}
