# Architecture

## Labrinth

Labrinth is the backend API service for Modrinth.

### Testing

Before a pull request can be opened, run `cargo clippy -p labrinth --all-targets` and make sure there are ZERO warnings, otherwise CI will fail.

Use `cargo test -p labrinth --all-targets` to test your changes. All tests must pass, otherwise CI will fail.

Read the root `docker-compose.yml` to see what running services are available while developing. Use `docker exec` to access these services.

### Clickhouse

Use `docker exec labrinth-clickhouse clickhouse-client` to access the Clickhouse instance. We use the `staging_ariadne` database to store data in testing.

### Postgres

Use `docker exec labrinth-postgres psql -U postgres` to access the PostgreSQL instance.
