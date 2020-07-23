use log::info;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    info!("Initializing database connection");

    let database_url = dotenv::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
