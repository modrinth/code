impl crate::Database for sqlx::Postgres {
    const SYSTEM: &'static str = "postgresql";
}
