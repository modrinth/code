impl crate::Database for sqlx::Sqlite {
    const SYSTEM: &'static str = "sqlite";

    type ConnectionRef<'a> = &'a mut sqlx::SqliteConnection;

    fn cast_connection<'c>(
        conn: &'c mut <Self as sqlx::Database>::Connection,
    ) -> Self::ConnectionRef<'c> {
        conn
    }
}
