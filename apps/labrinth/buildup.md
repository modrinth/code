# Labrinth (API)

This project is part of our [monorepo](https://github.com/modrinth/modrinth); you can find it in the `apps/labrinth` directory.

[`labrinth`](https://github.com/modrinth/labrinth) is the Rust-based backend serving Modrinth’s API with the help of the [Actix](https://actix.rs) framework. To get started with a labrinth instance, install Docker, docker-compose (which comes with Docker), and [Rust](https://www.rust-lang.org). The initial startup can be done simply with the command:

```bash
docker-compose up
```

or

```bash
docker compose up
```

(Compose V2 and later). This will deploy a PostgreSQL database on port 5432 and a MeiliSearch instance on port 7700.

To run the API itself, use the `cargo run` command; this will deploy the API on port 8000.

## Basic Setup

1. Copy the `.env.local` file to `.env`.
2. Install the sqlx CLI:

```bash
cargo install --git https://github.com/launchbadge/sqlx sqlx-cli --no-default-features --features postgres,rustls
```

3. Create the database and perform all migrations:

```bash
sqlx database setup
```

4. (Linux only) Install the OpenSSL library:
   On Debian-based systems:

```bash
sudo apt-get install pkg-config libssl-dev
```

## Seeding the Database

To enable labrinth to create a project, add:

1. An entry in the `loaders` table.
2. An entry in the `loaders_project_types` table.

Example using psql:

```bash
psql --host=localhost --port=5432 -U <username> -W
```

Default password: `labrinth`. Then run:

```sql
INSERT INTO loaders VALUES (0, 'placeholder_loader');

INSERT INTO loaders_project_types VALUES (0, 1); -- modloader id, supported type id

INSERT INTO categories VALUES (0, 'placeholder_category', 1); -- category id, category, project type id
```

This initializes a modloader `placeholder_loader` (id 0) supporting mods only, and a category `placeholder_category` (id 0) for mods only. To also support modpacks:

```sql
INSERT INTO loaders_project_types VALUES (0, 2);

INSERT INTO categories VALUES (0, 'placeholder_category', 2);
```

More example SQL is in `apps/labrinth/tests/files/dummy_data.sql`.

## Configuration (.env & CLI Options)

Configuration is done at runtime using [dotenvy](https://crates.io/crates/dotenvy) and the `.env` file. Variables:

### Basic Configuration

* `DEBUG`: Whether debugging tools should be enabled
* `RUST_LOG`: What to log (from rust’s [env-logger](https://github.com/env-logger)); default `info,sqlx::query=warn`
* `SITE_URL`: Main URL for CORS
* `CDN_URL`: Base URL for CDN uploads
* `MODERATION_DISCORD_WEBHOOK`: Discord webhook URL for pending projects
* `CLOUDFLARE_INTEGRATION`: Enable Cloudflare spam protection
* `DATABASE_URL`: PostgreSQL connection URL
* `DATABASE_MIN_CONNECTIONS`: Min DB connections
* `DATABASE_MAX_CONNECTIONS`: Max DB connections
* `MEILISEARCH_ADDR`: MeiliSearch URL
* `MEILISEARCH_KEY`: Name for MeiliSearch
* `BIND_ADDR`: Server bind address (IPv4/IPv6)
* `MOCK_FILE_PATH`: Path for file storage (required)
* `SMTP_USERNAME`: SMTP auth username
* `SMTP_PASSWORD`: SMTP auth password
* `SMTP_HOST`: SMTP server hostname/IP
* `SMTP_PORT`: SMTP server port (25, 465, 587)
* `SMTP_TLS`: SMTP TLS mode (`none`, `opportunistic_start_tls`, `requires_start_tls`, `tls`)

### CDN Options

* `STORAGE_BACKEND`: `local` (default) or `s3`

S3 options:

* `S3_ACCESS_TOKEN`
* `S3_SECRET`
* `S3_URL`
* `S3_REGION`
* `S3_PUBLIC_BUCKET_NAME`
* `S3_PRIVATE_BUCKET_NAME`
* `S3_USES_PATH_STYLE_BUCKETS`

### Search, OAuth & Miscellaneous

* `LOCAL_INDEX_INTERVAL`: Seconds between reindexing DB (default `3600`)
* `VERSION_INDEX_INTERVAL`: Seconds between version reindexing (default `1800`)

OAuth options are self-explanatory. For help, contact us on [Discord](https://discord.modrinth.com).

* `RATE_LIMIT_IGNORE_IPS`: Array of IPs with lower rate limit factor

### Command Line Options

* `--skip-first-index`: Skip initial indexing
* `--reconfigure-indices`: Reset MeiliSearch settings and exit
* `--reset-indices`: Reset MeiliSearch indices and exit (clears all data)

## Ready to Open a PR?

Ensure you have run:

* `cargo fmt`
* `cargo clippy`
* `cargo check`
* `cargo sqlx prepare`

> **Note:** If `sqlx` reports “no queries found” after running `cargo sqlx prepare`, ensure your `sqlx-cli` version matches the `sqlx` version used in labrinth.

[Edit page](https://github.com/modrinth/modrinth/edit/main/apps/labrinth)
