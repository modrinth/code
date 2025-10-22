use actix_web::guard::GuardContext;
use actix_web::http::header::X_FORWARDED_FOR;

pub const ADMIN_KEY_HEADER: &str = "Modrinth-Admin";
pub const MEDAL_KEY_HEADER: &str = "X-Medal-Access-Key";
pub const EXTERNAL_NOTIFICATION_KEY_HEADER: &str = "External-Notification-Key";

pub fn admin_key_guard(ctx: &GuardContext) -> bool {
    let admin_key = std::env::var("LABRINTH_ADMIN_KEY").expect(
        "No admin key provided, this should have been caught by check_env_vars",
    );
    ctx.head()
        .headers()
        .get(ADMIN_KEY_HEADER)
        .is_some_and(|it| it.as_bytes() == admin_key.as_bytes())
}

pub fn medal_key_guard(ctx: &GuardContext) -> bool {
    let maybe_medal_key = dotenvy::var("LABRINTH_MEDAL_KEY").ok();

    match maybe_medal_key {
        None => false,
        Some(medal_key) => ctx
            .head()
            .headers()
            .get(MEDAL_KEY_HEADER)
            .is_some_and(|it| it.as_bytes() == medal_key.as_bytes()),
    }
}

pub fn external_notification_key_guard(ctx: &GuardContext) -> bool {
    let maybe_external_notification_key =
        dotenvy::var("LABRINTH_EXTERNAL_NOTIFICATION_KEY").ok();

    match maybe_external_notification_key {
        None => false,
        Some(external_notification_key) => ctx
            .head()
            .headers()
            .get(EXTERNAL_NOTIFICATION_KEY_HEADER)
            .is_some_and(|it| {
                it.as_bytes() == external_notification_key.as_bytes()
            }),
    }
}

pub fn internal_network_guard(ctx: &GuardContext) -> bool {
    ctx.head()
		.peer_addr
		.is_some_and(|sock| matches!(sock.ip().to_canonical(), std::net::IpAddr::V4(v4) if v4.is_private()))
		&& ctx.head().headers().get(X_FORWARDED_FOR).is_none()
}
