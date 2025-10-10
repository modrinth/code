# Architecture

## Labrinth

Labrinth is the backend API service for Modrinth.

### Testing

Before a pull request can be opened, run `cargo clippy -p labrinth --all-targets` and make sure there are ZERO warnings, otherwise CI will fail.

Use `cargo test -p labrinth --all-targets` to test your changes. All tests must pass, otherwise CI will fail.

To prepare the sqlx cache, cd into `apps/labrinth` and run `cargo sqlx prepare`. Make sure to NEVER run `cargo sqlx prepare --workspace`.

Read the root `docker-compose.yml` to see what running services are available while developing. Use `docker exec` to access these services.

When the user refers to "performing pre-PR checks", do the following:
- Run clippy as described above
- DO NOT run tests unless explicitly requested (they take a long time)
- Prepare the sqlx cache

### Clickhouse

Use `docker exec labrinth-clickhouse clickhouse-client` to access the Clickhouse instance. We use the `staging_ariadne` database to store data in testing.

### Postgres

Use `docker exec labrinth-postgres psql -U postgres` to access the PostgreSQL instance.
