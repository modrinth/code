> [!NOTE]
>
> This is a vendored version of [`sqlx-tracing`](https://github.com/jdrouet/sqlx-tracing/) with modifications for our own purposes.
>
> This directory is licensed under the same license as the original project.

# sqlx-tracing

**sqlx-tracing** is a Rust library that provides OpenTelemetry-compatible tracing for SQLx database operations. It wraps SQLx connection pools and queries with tracing spans, enabling detailed observability of database interactions in distributed systems.

## Features

- **Automatic Tracing**: All SQLx queries executed through the provided pool are traced using [tracing](https://docs.rs/tracing) spans.
- **OpenTelemetry Integration**: Traces are compatible with OpenTelemetry, making it easy to export to collectors and observability platforms.
- **Error Recording**: Errors are automatically annotated with kind, message, and stacktrace in the tracing span.
- **Returned Rows**: The number of rows returned by queries is recorded for observability.
- **Database Agnostic**: Supports both PostgreSQL and SQLite via feature flags.
- **Macros**: Includes a macro for consistent span creation around queries.

## Usage

Add `sqlx-tracing` to your `Cargo.toml`:

```toml
[dependencies]
sqlx-tracing = "0.1"
sqlx = { version = "0.8", default-features = false, features = ["derive"] }
tracing = "0.1"
```

Enable the desired database feature:

- For PostgreSQL: `features = ["postgres"]`
- For SQLite: `features = ["sqlite"]`

Wrap your SQLx pool:

```rust,ignore
let pool = sqlx::PgPool::connect(&url).await?;
// the attributes will be resolved from the url
let traced_pool = sqlx_tracing::Pool::from(pool);
// or manually overwrite them
let traced_pool = sqlx_tracing::PoolBuilder::from(pool)
    .with_name("my-domain-database")
    .with_database("database")
    .with_host("somewhere")
    .with_port(1234)
    .build();
```

Use the traced pool as you would a normal SQLx pool:

```rust,ignore
let result: Option<i32> = sqlx::query_scalar("select 1")
    .fetch_optional(traced_pool)
    .await?;
```

This works also with pool connections

```rust,ignore
let mut conn = traced_pool.acquire().await?;
let result: Option<i32> = sqlx::query_scalar("select 1")
    .fetch_optional(&mut conn)
    .await?;
```

And transactions

```rust,ignore
let mut tx = traced_pool.begin().await?;
let result: Option<i32> = sqlx::query_scalar("select 1")
    .fetch_optional(&mut tx.executor())
    .await?;
```

## OpenTelemetry Integration

To export traces, set up an OpenTelemetry collector and configure the tracing subscriber with the appropriate layers. See the `tests/common.rs` for a full example using `opentelemetry`, `opentelemetry-otlp`, and `tracing-opentelemetry`.

## Testing

Integration tests are provided for both PostgreSQL and SQLite, using [testcontainers](https://docs.rs/testcontainers) and a local OpenTelemetry collector.

## License

Licensed under MIT.

## Contributing

Contributions and issues are welcome! Please open a PR or issue on GitHub.
