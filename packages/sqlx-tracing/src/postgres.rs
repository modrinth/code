impl crate::prelude::Database for sqlx::Postgres {
    const SYSTEM: &'static str = "postgresql";
}
