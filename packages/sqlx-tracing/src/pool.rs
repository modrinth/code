use futures::{StreamExt, TryStreamExt, future::BoxFuture};
use tracing::Instrument;

use crate::AnyConnection;

impl<'c, DB> crate::Acquire<'c> for &'c crate::Pool<DB>
where
    DB: crate::Database,
{
    type Database = DB;

    fn acquire(
        self,
    ) -> BoxFuture<'c, Result<AnyConnection<'c, DB>, sqlx::Error>> {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.acquire", attrs);
        let fut = self.inner.acquire();
        let fut = async move {
            let conn = fut.await.inspect_err(crate::span::record_error)?;
            let conn = crate::PoolConnection {
                inner: conn,
                attributes: self.attributes.clone(),
            };
            let conn = AnyConnection::Pool(conn);
            Ok::<_, sqlx::Error>(conn)
        };
        Box::pin(fut.instrument(span))
    }

    fn begin(
        self,
    ) -> BoxFuture<
        'c,
        Result<crate::Transaction<'c, Self::Database>, sqlx::Error>,
    > {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.begin", attrs);
        let fut = self.inner.begin();
        Box::pin(
            async move {
                let txn = fut.await.inspect_err(crate::span::record_error)?;
                let txn = crate::Transaction {
                    inner: txn,
                    attributes: self.attributes.clone(),
                };
                Ok::<_, sqlx::Error>(txn)
            }
            .instrument(span),
        )
    }
}

impl<DB: crate::Database> crate::Pool<DB> {
    /// Retrieves a connection and immediately begins a new transaction.
    ///
    /// The returned [`Transaction`] is instrumented for tracing.
    ///
    /// [`Transaction`]: crate::Transaction
    pub async fn begin<'c>(
        &'c self,
    ) -> Result<crate::Transaction<'c, DB>, sqlx::Error> {
        self.inner.begin().await.map(|inner| crate::Transaction {
            inner,
            attributes: self.attributes.clone(),
        })
    }

    /// Acquires a pooled connection, instrumented for tracing.
    pub async fn acquire(
        &self,
    ) -> Result<crate::PoolConnection<DB>, sqlx::Error> {
        self.inner
            .acquire()
            .await
            .map(|inner| crate::PoolConnection {
                attributes: self.attributes.clone(),
                inner,
            })
    }
}

impl<'p, DB> sqlx::Executor<'p> for &crate::Pool<DB>
where
    DB: crate::Database,
    for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
{
    type Database = DB;

    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> futures::future::BoxFuture<
        'e,
        Result<sqlx::Describe<Self::Database>, sqlx::Error>,
    > {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.describe", attrs, sql);
        let fut = self.inner.describe(sql);
        Box::pin(
            async move { fut.await.inspect_err(crate::span::record_error) }
                .instrument(span),
        )
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
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.execute", attrs, sql);
        let fut = self.inner.execute(query);
        Box::pin(
            async move { fut.await.inspect_err(crate::span::record_error) }
                .instrument(span),
        )
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
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.execute_many", attrs, sql);
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
    ) -> futures::stream::BoxStream<
        'e,
        Result<<Self::Database as sqlx::Database>::Row, sqlx::Error>,
    >
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch", attrs, sql);
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
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_all", attrs, sql);
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
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_all", attrs, sql);
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
    ) -> futures::future::BoxFuture<
        'e,
        Result<<Self::Database as sqlx::Database>::Row, sqlx::Error>,
    >
    where
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_one", attrs, sql);
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
    {
        let sql = query.sql();
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.fetch_optional", attrs, sql);
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
    > {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.prepare", attrs, query);
        let fut = self.inner.prepare(query);
        Box::pin(
            async move { fut.await.inspect_err(crate::span::record_error) }
                .instrument(span),
        )
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
    ) -> futures::future::BoxFuture<
        'e,
        Result<<Self::Database as sqlx::Database>::Statement<'q>, sqlx::Error>,
    > {
        let attrs = &self.attributes;
        let span = crate::instrument!("sqlx.prepare_with", attrs, sql);
        let fut = self.inner.prepare_with(sql, parameters);
        Box::pin(
            async move { fut.await.inspect_err(crate::span::record_error) }
                .instrument(span),
        )
    }
}
