impl crate::Database for sqlx::Postgres {
    const SYSTEM: &'static str = "postgresql";

    type ConnectionRef<'a> = &'a mut sqlx::PgConnection;

    fn cast_connection<'c>(
        conn: &'c mut <Self as sqlx::Database>::Connection,
    ) -> Self::ConnectionRef<'c> {
        conn
    }

    // fn cast_pool_connection<'c>(
    //     conn: &'c mut PoolConnection<Self>,
    // ) -> Self::PoolConnection<'c> {
    //     &mut conn.inner
    // }

    // fn cast_raw_connection<'c>(
    //     conn: &'c mut <Self as sqlx::Database>::Connection,
    // ) -> Self::RawConnection<'c> {
    //     conn
    // }
}
