use crate::database::Executor;
use eyre::{Context, Result};
use sqlx::AssertSqlSafe;

use crate::database::{PgPool, models::DBUserId};

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

/// Mark a user's email as verified without using the email verification flow.
///
/// # Errors
///
/// Errors if the user does not exist or could not be updated.
pub async fn mark_email_verified(db: &PgPool, user_id: DBUserId) -> Result<()> {
    let result = sqlx::query!(
        "UPDATE users SET email_verified = TRUE WHERE id = $1",
        user_id.0,
    )
    .execute(db)
    .await
    .wrap_err("failed to mark user email as verified")?;

    eyre::ensure!(result.rows_affected() == 1, "user does not exist");

    Ok(())
}

/// Initialize fixture project state without exercising the public PATCH endpoint.
///
/// # Errors
///
/// Errors if the project cannot be loaded, updated, or invalidated in cache.
pub async fn initialize_project_state(
    response: &actix_web::dev::ServiceResponse,
    id_or_slug: &str,
    review_status: crate::models::v3::projects::ProjectReviewStatus,
    visibility: Option<crate::models::v3::projects::ProjectVisibility>,
) -> Result<()> {
    use crate::database::models::project_item::DBProject;
    use crate::models::v3::projects::ReviewStatusProvenance;
    use actix_web::web::Data;

    let pool = response
        .request()
        .app_data::<Data<PgPool>>()
        .ok_or_else(|| eyre::eyre!("missing test database pool"))?;
    let redis = response
        .request()
        .app_data::<Data<xredis::RedisPool>>()
        .ok_or_else(|| eyre::eyre!("missing test Redis pool"))?;
    let project =
        Box::pin(DBProject::get(id_or_slug, pool.get_ref(), redis.get_ref()))
            .await?
            .ok_or_else(|| eyre::eyre!("fixture project does not exist"))?;
    let mut transaction = pool.begin().await?;
    DBProject::update_review_status(
        project.inner.id,
        review_status,
        None,
        &ReviewStatusProvenance::System {
            reason: "initialize test fixture".to_string(),
        },
        &mut transaction,
    )
    .await?;
    if let Some(visibility) = visibility {
        DBProject::update_visibility(
            project.inner.id,
            visibility,
            None,
            &mut transaction,
        )
        .await?;
    }
    transaction.commit().await?;
    DBProject::clear_cache(
        project.inner.id,
        project.inner.slug,
        None,
        redis.get_ref(),
    )
    .await?;

    Ok(())
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

/// Initialize the Minecraft metadata catalog in an isolated test database.
///
/// # Errors
///
/// Errors if the fixture could not be applied.
pub async fn add_minecraft_meta_dummy_data(db: &PgPool) -> Result<()> {
    db.execute(include_str!("../../fixtures/minecraft-meta-dummy-data.sql"))
        .await
        .wrap_err("failed to add Minecraft metadata dummy data")?;

    Ok(())
}

/// Initialize a database with dummy fixture data.
///
/// # Errors
///
/// Errors if the fixture could not be applied.
pub async fn add_dummy_data(db: &PgPool) -> Result<()> {
    // Fixture SQL only substitutes numeric values and hard-coded, quote-free PATs.
    db.execute(AssertSqlSafe(
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
            ),
    ))
    .await
    .wrap_err("failed to add dummy data")?;

    for user_id in [
        user_id::ADMIN,
        user_id::MODERATOR,
        user_id::USER,
        user_id::FRIEND,
        user_id::ENEMY,
    ] {
        mark_email_verified(db, user_id).await?;
    }

    Ok(())
}
