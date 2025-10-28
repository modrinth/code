use eyre::{Context, Result};
use sqlx::PgPool;

pub mod pat {
    pub const ADMIN: &str = "mrp_patadmin";
    pub const MODERATOR: &str = "mrp_patmoderator";
    pub const USER: &str = "mrp_patuser";
    pub const FRIEND: &str = "mrp_patfriend";
    pub const ENEMY: &str = "mrp_patenemy";
}

pub mod user_id {
    use crate::database::models::DBUserId;

    pub const ADMIN: DBUserId = DBUserId(1);
    pub const MODERATOR: DBUserId = DBUserId(2);
    pub const USER: DBUserId = DBUserId(3);
    pub const FRIEND: DBUserId = DBUserId(4);
    pub const ENEMY: DBUserId = DBUserId(5);
}

pub async fn add_dummy_data(db: &PgPool) -> Result<()> {
    sqlx::query(
        include_str!("../../fixtures/dummy_data.sql")
            //
            .replace("{{user_id::ADMIN}}", &user_id::ADMIN.0.to_string())
            .replace(
                "{{user_id::MODERATOR}}",
                &user_id::MODERATOR.0.to_string(),
            )
            .replace("{{user_id::USER}}", &user_id::USER.0.to_string())
            .replace("{{user_id::FRIEND}}", &user_id::FRIEND.0.to_string())
            .replace("{{user_id::ENEMY}}", &user_id::ENEMY.0.to_string())
            //
            .replace("{{pat::ADMIN}}", pat::ADMIN)
            .replace("{{pat::MODERATOR}}", pat::MODERATOR)
            .replace("{{pat::USER}}", pat::USER)
            .replace("{{pat::FRIEND}}", pat::FRIEND)
            .replace("{{pat::ENEMY}}", pat::ENEMY)
            //
            .replace(
                "{{all_scopes}}",
                &crate::models::pats::Scopes::all().bits().to_string(),
            )
            .as_str(),
    )
    .execute(db)
    .await
    .wrap_err("failed to add dummy data")?;

    Ok(())
}
