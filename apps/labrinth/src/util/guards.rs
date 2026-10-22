use actix_web::guard::GuardContext;
use actix_web::http::header::X_FORWARDED_FOR;
use subtle::ConstantTimeEq;

use crate::env::ENV;

pub const ADMIN_KEY_HEADER: &str = "Modrinth-Admin";
pub const MEDAL_KEY_HEADER: &str = "X-Medal-Access-Key";
pub const EXTERNAL_NOTIFICATION_KEY_HEADER: &str = "External-Notification-Key";
pub const SUBSCRIPTIONS_KEY_HEADER: &str = "Modrinth-Subscriptions-Key";

/// Checks that the header `name` matches `key`, comparing in constant time to
/// avoid leaking the key through timing side channels.
fn header_matches_key(ctx: &GuardContext, name: &str, key: &str) -> bool {
    ctx.head()
        .headers()
        .get(name)
        .is_some_and(|it| it.as_bytes().ct_eq(key.as_bytes()).into())
}

pub fn admin_key_guard(ctx: &GuardContext) -> bool {
    header_matches_key(ctx, ADMIN_KEY_HEADER, &ENV.LABRINTH_ADMIN_KEY)
}

pub fn medal_key_guard(ctx: &GuardContext) -> bool {
    header_matches_key(ctx, MEDAL_KEY_HEADER, &ENV.LABRINTH_MEDAL_KEY)
}

pub fn external_notification_key_guard(ctx: &GuardContext) -> bool {
    header_matches_key(
        ctx,
        EXTERNAL_NOTIFICATION_KEY_HEADER,
        &ENV.LABRINTH_EXTERNAL_NOTIFICATION_KEY,
    )
}

pub fn subscriptions_key_guard(ctx: &GuardContext) -> bool {
    // Ensure the subs key is set and at least 32 characters
    if ENV.LABRINTH_SUBSCRIPTIONS_KEY.chars().count() < 32 {
        return false;
    }

    header_matches_key(
        ctx,
        SUBSCRIPTIONS_KEY_HEADER,
        &ENV.LABRINTH_SUBSCRIPTIONS_KEY,
    )
}

pub fn internal_network_guard(ctx: &GuardContext) -> bool {
    ctx.head()
		.peer_addr
		.is_some_and(|sock| matches!(sock.ip().to_canonical(), std::net::IpAddr::V4(v4) if v4.is_private()))
		&& ctx.head().headers().get(X_FORWARDED_FOR).is_none()
}
