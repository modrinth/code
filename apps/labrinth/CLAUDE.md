# Labrinth

Labrinth is the backend API service for Modrinth, written in Rust.

## Pre-PR Checks

When the user refers to "perform[ing] pre-PR checks", do the following:

- Run `cargo clippy -p labrinth --all-targets` — there must be ZERO warnings, otherwise CI will fail
- DO NOT run tests unless explicitly requested (they take a long time)
- Prepare the sqlx cache: cd into `apps/labrinth` and run `cargo sqlx prepare`
	- NEVER run `cargo sqlx prepare --workspace`

## Testing

- Run `cargo test -p labrinth --all-targets` to test your changes — all tests must pass

## Local Services

- Read the root `docker-compose.yml` to see what running services are available while developing
- Use `docker exec` to access these services

### Clickhouse

- Access: `docker exec labrinth-clickhouse clickhouse-client`
- Database: `staging_ariadne`

### Postgres

- Access: `docker exec labrinth-postgres psql -U labrinth -d labrinth -c "<query>"`
