use eyre::{Context, Result};
use sqlx::{Executor, PgPool};

/// Static personal access token for use in [`AppendPat`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pat(pub &'static str);

/// Dummy [`Pat`]s.
#[allow(missing_docs, reason = "self-explanatory")]
pub mod pat {
    use super::Pat;

    pub const ADMIN: Pat = Pat("mrp_patadmin");
    pub const MODERATOR: Pat = Pat("mrp_patmoderator");
    pub const USER: Pat = Pat("mrp_patuser");
    pub const FRIEND: Pat = Pat("mrp_patfriend");
    pub const ENEMY: Pat = Pat("mrp_patenemy");
}

/// See [`AppendPat::append_pat`].
pub trait AppendPat {
    /// Appends a [`Pat`] authorization token to an
    /// [`actix_web::test::TestRequest`].
    #[must_use]
    fn append_pat(self, pat: Pat) -> Self;
}

impl AppendPat for actix_web::test::TestRequest {
    fn append_pat(self, pat: Pat) -> Self {
        self.append_header(("Authorization", pat.0))
    }
}

/// Dummy [`DBUserId`]s.
///
/// [`DBUserId`]: crate::database::models::DBUserId
pub mod user_id {
    use crate::database::models::DBUserId;

    pub const ADMIN: DBUserId = DBUserId(1);
    pub const MODERATOR: DBUserId = DBUserId(2);
    pub const USER: DBUserId = DBUserId(3);
    pub const FRIEND: DBUserId = DBUserId(4);
    pub const ENEMY: DBUserId = DBUserId(5);
}

/// Initialize a database with dummy fixture data.
///
/// # Errors
///
/// Errors if the fixture could not be applied.
pub async fn add_dummy_data(db: &PgPool) -> Result<()> {
    db.execute(
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
            .replace("{{pat::ADMIN}}", pat::ADMIN.0)
            .replace("{{pat::MODERATOR}}", pat::MODERATOR.0)
            .replace("{{pat::USER}}", pat::USER.0)
            .replace("{{pat::FRIEND}}", pat::FRIEND.0)
            .replace("{{pat::ENEMY}}", pat::ENEMY.0)
            //
            .replace(
                "{{all_scopes}}",
                &crate::models::pats::Scopes::all().bits().to_string(),
            )
            .as_str(),
    )
    .await
    .wrap_err("failed to add dummy data")?;

    Ok(())
}
