use crate::{AnyConnection, Database};

impl<'c, 's, DB> sqlx::Executor<'s> for &'s mut AnyConnection<'c, DB>
where
    DB: Database,
    // I attempted to have `DB::ConnectionRef<'c>` unify to `&'c mut DB::Connection`.
    // This *can* be unified apparently, but we can't actually use the fact that
    // `DB::ConnectionRef<'c>: sqlx::Executor` if we do this.
    // So, we need a casting function in `crate::Database`.
    // Maybe this can be revisited sometime to not require the casting fn.
    //
    // for<'a> DB: Database<ConnectionRef<'a> = &'a mut <DB as sqlx::Database>::Connection>,
{
    type Database = DB;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::stream::BoxStream<
        'e,
        Result<
            sqlx::Either<
                <Self::Database as sqlx::Database>::QueryResult,
                <Self::Database as sqlx::Database>::Row,
            >,
            sqlx::Error,
        >,
    >
    where
        's: 'e,
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        match self {
            AnyConnection::Pool(pool) => {
                DB::cast_connection(&mut pool.inner).fetch_many(query)
            }
            AnyConnection::Raw(conn) => {
                DB::cast_connection(conn.inner).fetch_many(query)
            }
        }
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<
        'e,
        Result<Option<<Self::Database as sqlx::Database>::Row>, sqlx::Error>,
    >
    where
        's: 'e,
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        match self {
            AnyConnection::Pool(pool) => {
                DB::cast_connection(&mut pool.inner).fetch_optional(query)
            }
            AnyConnection::Raw(conn) => {
                DB::cast_connection(conn.inner).fetch_optional(query)
            }
        }
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
    ) -> futures::future::BoxFuture<
        'e,
        Result<<Self::Database as sqlx::Database>::Statement<'q>, sqlx::Error>,
    >
    where
        's: 'e,
    {
        match self {
            AnyConnection::Pool(pool) => DB::cast_connection(&mut pool.inner)
                .prepare_with(sql, parameters),
            AnyConnection::Raw(conn) => {
                DB::cast_connection(conn.inner).prepare_with(sql, parameters)
            }
        }
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> futures::future::BoxFuture<
        'e,
        Result<sqlx::Describe<Self::Database>, sqlx::Error>,
    >
    where
        's: 'e,
    {
        match self {
            AnyConnection::Pool(pool) => {
                DB::cast_connection(&mut pool.inner).describe(sql)
            }
            AnyConnection::Raw(conn) => {
                DB::cast_connection(conn.inner).describe(sql)
            }
        }
    }
}
