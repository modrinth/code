#![doc = include_str!("../README.md")]

use std::sync::Arc;

use derive_more::{Deref, DerefMut};
use futures::future::BoxFuture;

mod any_connection;
mod connection;
mod pool;
pub(crate) mod span;
mod transaction;

pub use sqlx::Executor;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

/// Attributes describing the database connection and context.
/// Used for span enrichment and attribute propagation.
#[derive(Debug, Default)]
struct Attributes {
    name: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
}

pub trait Database: sqlx::Database {
    const SYSTEM: &'static str;

    /// Defines the type of reference to a database connection, equivalent to
    /// `&'c mut <Self as sqlx::Database>::Connection`.
    ///
    /// But we can't actually use the `sqlx::Database` named connection type,
    /// since we can't statically prove that it implements `sqlx::Executor`.
    /// Even if we unify the two types (see `any_connection.rs`), we can't use
    /// connection refs as an executor. So we need this intermediate associated
    /// type.
    type ConnectionRef<'c>: sqlx::Executor<'c, Database = Self>;

    /// Casts a `&'c mut Self::Connection` to a `Self::ConnectionRef<'c>`.
    ///
    /// This should just return `conn`.
    fn cast_connection<'c>(
        conn: &'c mut <Self as sqlx::Database>::Connection,
    ) -> Self::ConnectionRef<'c>;
}

/// Builder for constructing a [`Pool`] with custom attributes.
///
/// Allows setting database name, host, port, and other identifying information
/// for tracing purposes.
#[derive(Debug)]
pub struct PoolBuilder<DB: Database> {
    pool: sqlx::Pool<DB>,
    attributes: Attributes,
}

// this is required because `pool.connect_options().to_url_lossy()` panics with sqlite
#[cfg(feature = "postgres")]
impl From<sqlx::Pool<sqlx::Postgres>> for PoolBuilder<sqlx::Postgres> {
    /// Create a new builder from an existing SQLx pool.
    fn from(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        use sqlx::ConnectOptions;

        let url = pool.connect_options().to_url_lossy();
        let attributes = Attributes {
            name: None,
            host: url.host_str().map(String::from),
            port: url.port(),
            database: url
                .path_segments()
                .and_then(|mut segments| segments.next().map(String::from)),
        };
        Self { pool, attributes }
    }
}

// this is required because `pool.connect_options().to_url_lossy()` panics with sqlite
#[cfg(feature = "sqlite")]
impl From<sqlx::Pool<sqlx::Sqlite>> for PoolBuilder<sqlx::Sqlite> {
    /// Create a new builder from an existing SQLx pool.
    fn from(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        let attributes = Attributes {
            name: None,
            host: pool
                .connect_options()
                .get_filename()
                .to_str()
                .map(String::from),
            port: None,
            database: None,
        };
        Self { pool, attributes }
    }
}

impl<DB: Database> PoolBuilder<DB> {
    /// Set a custom name for the pool (for peer.service attribute).
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.attributes.name = Some(name.into());
        self
    }

    /// Set the database name attribute.
    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.attributes.database = Some(database.into());
        self
    }

    /// Set the host attribute.
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.attributes.host = Some(host.into());
        self
    }

    /// Set the port attribute.
    pub fn with_port(mut self, port: u16) -> Self {
        self.attributes.port = Some(port);
        self
    }

    /// Build the [`Pool`] with the configured attributes.
    pub fn build(self) -> Pool<DB> {
        Pool {
            inner: self.pool,
            attributes: Arc::new(self.attributes),
        }
    }
}

/// An asynchronous pool of SQLx database connections with tracing instrumentation.
///
/// Wraps a SQLx [`Pool`] and propagates tracing attributes to all acquired connections.
#[derive(Debug, Deref, DerefMut)]
pub struct Pool<DB: Database> {
    #[deref]
    #[deref_mut]
    inner: sqlx::Pool<DB>,
    attributes: Arc<Attributes>,
}

// manually impl `Clone` because `DB` may not be `Clone`
impl<DB: Database> Clone for Pool<DB> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            attributes: self.attributes.clone(),
        }
    }
}

impl<DB> From<sqlx::Pool<DB>> for Pool<DB>
where
    DB: Database,
    PoolBuilder<DB>: From<sqlx::Pool<DB>>,
{
    /// Convert a SQLx [`Pool`] into a tracing-instrumented [`Pool`].
    fn from(inner: sqlx::Pool<DB>) -> Self {
        PoolBuilder::from(inner).build()
    }
}

/// Wrapper for a mutable SQLx connection reference with tracing attributes.
///
/// Used internally for transaction and pool connection executors.
pub struct Connection<'c, DB: Database> {
    inner: &'c mut DB::Connection,
    attributes: Arc<Attributes>,
}

impl<'c, DB: Database> std::fmt::Debug for Connection<'c, DB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection").finish_non_exhaustive()
    }
}

/// A pooled SQLx connection instrumented for tracing.
///
/// Implements [`sqlx::Executor`] and propagates tracing attributes.
#[derive(Debug)]
pub struct PoolConnection<DB: Database> {
    inner: sqlx::pool::PoolConnection<DB>,
    attributes: Arc<Attributes>,
}

/// An in-progress database transaction or savepoint, instrumented for tracing.
///
/// Wraps a SQLx [`Transaction`] and propagates tracing attributes.
#[derive(Debug)]
pub struct Transaction<'c, DB: Database> {
    inner: sqlx::Transaction<'c, DB>,
    attributes: Arc<Attributes>,
}

/// Acquire connections or transactions from a database in a generic way.
///
/// Equivalent of [`sqlx::Acquire`] with tracing.
pub trait Acquire<'c> {
    type Database: Database;

    fn acquire(
        self,
    ) -> BoxFuture<'c, Result<AnyConnection<'c, Self::Database>, sqlx::Error>>;

    fn begin(
        self,
    ) -> BoxFuture<'c, Result<Transaction<'c, Self::Database>, sqlx::Error>>;
}

#[derive(Debug)]
pub enum AnyConnection<'c, DB: Database> {
    Pool(PoolConnection<DB>),
    Raw(Connection<'c, DB>),
}
