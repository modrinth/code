use actix_web::guard::GuardContext;

pub const ADMIN_KEY_HEADER: &str = "Modrinth-Admin";
pub const MEDAL_KEY_HEADER: &str = "X-Medal-Access-Key";

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
