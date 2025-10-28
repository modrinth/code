use eyre::{Context, Result};
use sqlx::PgPool;

use crate::models::pats::Scopes;

pub async fn add_dummy_data(db: &PgPool) -> Result<()> {
    sqlx::query(
        include_str!("../../fixtures/dummy_data.sql")
            .replace("$1", &Scopes::all().bits().to_string())
            .as_str(),
    )
    .execute(db)
    .await
    .wrap_err("failed to add dummy data")?;

    Ok(())
}
