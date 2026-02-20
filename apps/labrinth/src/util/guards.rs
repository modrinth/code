use actix_web::guard::GuardContext;
use actix_web::http::header::X_FORWARDED_FOR;

use crate::env::ENV;

pub const ADMIN_KEY_HEADER: &str = "Modrinth-Admin";
pub const MEDAL_KEY_HEADER: &str = "X-Medal-Access-Key";
pub const EXTERNAL_NOTIFICATION_KEY_HEADER: &str = "External-Notification-Key";

pub fn admin_key_guard(ctx: &GuardContext) -> bool {
    ctx.head()
        .headers()
        .get(ADMIN_KEY_HEADER)
        .is_some_and(|it| it.as_bytes() == ENV.LABRINTH_ADMIN_KEY.as_bytes())
}

pub fn medal_key_guard(ctx: &GuardContext) -> bool {
    ctx.head()
        .headers()
        .get(MEDAL_KEY_HEADER)
        .is_some_and(|it| it.as_bytes() == ENV.LABRINTH_MEDAL_KEY.as_bytes())
}

pub fn external_notification_key_guard(ctx: &GuardContext) -> bool {
    ctx.head()
        .headers()
        .get(EXTERNAL_NOTIFICATION_KEY_HEADER)
        .is_some_and(|it| {
            it.as_bytes() == ENV.LABRINTH_EXTERNAL_NOTIFICATION_KEY.as_bytes()
        })
}

pub fn internal_network_guard(ctx: &GuardContext) -> bool {
    ctx.head()
		.peer_addr
		.is_some_and(|sock| matches!(sock.ip().to_canonical(), std::net::IpAddr::V4(v4) if v4.is_private()))
		&& ctx.head().headers().get(X_FORWARDED_FOR).is_none()
}
