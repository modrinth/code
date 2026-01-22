use futures::{StreamExt, TryStreamExt};
use tracing::Instrument;

impl<DB> AsMut<<DB as sqlx::Database>::Connection> for crate::PoolConnection<DB>
where
    DB: crate::prelude::Database + sqlx::Database,
{
    fn as_mut(&mut self) -> &mut <DB as sqlx::Database>::Connection {
        self.inner.as_mut()
    }
}

impl<'c, DB> sqlx::Executor<'c> for &'c mut crate::PoolConnection<DB>
where
    DB: crate::prelude::Database + sqlx::Database,
    for<'a> &'a mut DB::Connection: sqlx::Executor<'a, Database = DB>,
{
    type Database = DB;

    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> futures::future::BoxFuture<'e, Result<sqlx::Describe<Self::Database>, sqlx::Error>>
    where
        'c: 'e,
    {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.describe", sql, attrs);
        let fut = self.inner.as_mut().describe(sql);
        Box::pin(async move { fut.await.inspect_err(crate::span::record_error) }.instrument(span))
    }

    fn execute<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<
        'e,
        Result<<Self::Database as sqlx::Database>::QueryResult, sqlx::Error>,
    >
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.execute", sql, attrs);
        let fut = self.inner.execute(query);
        Box::pin(async move { fut.await.inspect_err(crate::span::record_error) }.instrument(span))
    }

    fn execute_many<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::stream::BoxStream<
        'e,
        Result<<Self::Database as sqlx::Database>::QueryResult, sqlx::Error>,
    >
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.execute_many", sql, attrs);
        let stream = self.inner.execute_many(query);
        use futures::StreamExt;
        Box::pin(
            stream
                .inspect(move |_| {
                    let _enter = span.enter();
                })
                .inspect_err(crate::span::record_error),
        )
    }

    fn fetch<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::stream::BoxStream<'e, Result<<Self::Database as sqlx::Database>::Row, sqlx::Error>>
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch", sql, attrs);
        let stream = self.inner.fetch(query);
        use futures::StreamExt;
        Box::pin(
            stream
                .inspect(move |_| {
                    let _enter = span.enter();
                })
                .inspect_err(crate::span::record_error),
        )
    }

    fn fetch_all<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<
        'e,
        Result<Vec<<Self::Database as sqlx::Database>::Row>, sqlx::Error>,
    >
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_all", sql, attrs);
        let fut = self.inner.fetch_all(query);
        Box::pin(
            async move {
                fut.await
                    .inspect(|res| {
                        let span = tracing::Span::current();
                        span.record("db.response.returned_rows", res.len());
                    })
                    .inspect_err(crate::span::record_error)
            }
            .instrument(span),
        )
    }

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
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_all", sql, attrs);
        let stream = self.inner.fetch_many(query);
        Box::pin(
            stream
                .inspect(move |_| {
                    let _enter = span.enter();
                })
                .inspect_err(crate::span::record_error),
        )
    }

    fn fetch_one<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<'e, Result<<Self::Database as sqlx::Database>::Row, sqlx::Error>>
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_one", sql, attrs);
        let fut = self.inner.fetch_one(query);
        Box::pin(
            async move {
                fut.await
                    .inspect(|_| {
                        tracing::Span::current().record("db.response.returned_rows", 1);
                    })
                    .inspect_err(crate::span::record_error)
            }
            .instrument(span),
        )
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<
        'e,
        Result<Option<<Self::Database as sqlx::Database>::Row>, sqlx::Error>,
    >
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_optional", sql, attrs);
        let fut = self.inner.fetch_optional(query);
        Box::pin(
            async move {
                fut.await
                    .inspect(|res| {
                        tracing::Span::current().record(
                            "db.response.returned_rows",
                            if res.is_some() { 1 } else { 0 },
                        );
                    })
                    .inspect_err(crate::span::record_error)
            }
            .instrument(span),
        )
    }

    fn prepare<'e, 'q: 'e>(
        self,
        query: &'q str,
    ) -> futures::future::BoxFuture<
        'e,
        Result<<Self::Database as sqlx::Database>::Statement<'q>, sqlx::Error>,
    >
    where
        'c: 'e,
    {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.prepare", query, attrs);
        let fut = self.inner.prepare(query);
        Box::pin(async move { fut.await.inspect_err(crate::span::record_error) }.instrument(span))
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
        'c: 'e,
    {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.prepare_with", sql, attrs);
        let fut = self.inner.prepare_with(sql, parameters);
        Box::pin(async move { fut.await.inspect_err(crate::span::record_error) }.instrument(span))
    }
}

impl<'c, DB> sqlx::Executor<'c> for &'c mut crate::Connection<'c, DB>
where
    DB: crate::prelude::Database + sqlx::Database,
    for<'a> &'a mut DB::Connection: sqlx::Executor<'a, Database = DB>,
{
    type Database = DB;

    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> futures::future::BoxFuture<'e, Result<sqlx::Describe<Self::Database>, sqlx::Error>>
    where
        'c: 'e,
    {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.describe", sql, attrs);
        let fut = self.inner.describe(sql);
        Box::pin(async move { fut.await.inspect_err(crate::span::record_error) }.instrument(span))
    }

    fn execute<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<
        'e,
        Result<<Self::Database as sqlx::Database>::QueryResult, sqlx::Error>,
    >
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.execute", sql, attrs);
        let fut = self.inner.execute(query);
        Box::pin(async move { fut.await.inspect_err(crate::span::record_error) }.instrument(span))
    }

    fn execute_many<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::stream::BoxStream<
        'e,
        Result<<Self::Database as sqlx::Database>::QueryResult, sqlx::Error>,
    >
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.execute_many", sql, attrs);
        let stream = self.inner.execute_many(query);
        use futures::StreamExt;
        Box::pin(
            stream
                .inspect(move |_| {
                    let _enter = span.enter();
                })
                .inspect_err(crate::span::record_error),
        )
    }

    fn fetch<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::stream::BoxStream<'e, Result<<Self::Database as sqlx::Database>::Row, sqlx::Error>>
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch", sql, attrs);
        let stream = self.inner.fetch(query);
        use futures::StreamExt;
        Box::pin(
            stream
                .inspect(move |_| {
                    let _enter = span.enter();
                })
                .inspect_err(crate::span::record_error),
        )
    }

    fn fetch_all<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<
        'e,
        Result<Vec<<Self::Database as sqlx::Database>::Row>, sqlx::Error>,
    >
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_all", sql, attrs);
        let fut = self.inner.fetch_all(query);
        Box::pin(
            async move {
                fut.await
                    .inspect(|res| {
                        let span = tracing::Span::current();
                        span.record("db.response.returned_rows", res.len());
                    })
                    .inspect_err(crate::span::record_error)
            }
            .instrument(span),
        )
    }

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
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_all", sql, attrs);
        let stream = self.inner.fetch_many(query);
        Box::pin(
            stream
                .inspect(move |_| {
                    let _enter = span.enter();
                })
                .inspect_err(crate::span::record_error),
        )
    }

    fn fetch_one<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<'e, Result<<Self::Database as sqlx::Database>::Row, sqlx::Error>>
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_one", sql, attrs);
        let fut = self.inner.fetch_one(query);
        Box::pin(
            async move {
                fut.await
                    .inspect(crate::span::record_one)
                    .inspect_err(crate::span::record_error)
            }
            .instrument(span),
        )
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<
        'e,
        Result<Option<<Self::Database as sqlx::Database>::Row>, sqlx::Error>,
    >
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
        'c: 'e,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_optional", sql, attrs);
        let fut = self.inner.fetch_optional(query);
        Box::pin(
            async move {
                fut.await
                    .inspect(crate::span::record_optional)
                    .inspect_err(crate::span::record_error)
            }
            .instrument(span),
        )
    }

    fn prepare<'e, 'q: 'e>(
        self,
        query: &'q str,
    ) -> futures::future::BoxFuture<
        'e,
        Result<<Self::Database as sqlx::Database>::Statement<'q>, sqlx::Error>,
    >
    where
        'c: 'e,
    {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.prepare", query, attrs);
        let fut = self.inner.prepare(query);
        Box::pin(async move { fut.await.inspect_err(crate::span::record_error) }.instrument(span))
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
        'c: 'e,
    {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.prepare_with", sql, attrs);
        let fut = self.inner.prepare_with(sql, parameters);
        Box::pin(async move { fut.await.inspect_err(crate::span::record_error) }.instrument(span))
    }
}
