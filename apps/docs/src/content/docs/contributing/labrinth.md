---
title: Labrinth (API)
description: Guide for contributing to Modrinth's backend
---

This project is part of our [monorepo](https://github.com/modrinth/code). You can find it in the `apps/labrinth` directory.

[labrinth] is the Rust-based backend serving Modrinth's API with the help of the [Actix](https://actix.rs) framework. To get started with a labrinth instance, install docker, docker-compose (which comes with Docker), and [Rust]. The initial startup can be done simply with the command `docker-compose up`, or with `docker compose up` (Compose V2 and later). That will deploy a PostgreSQL database on port 5432 and a MeiliSearch instance on port 7700. To run the API itself, you'll need to use the `cargo run` command, this will deploy the API on port 8000.

To get a basic configuration, copy the `.env.example` file to `.env`. Now, you'll have to install the sqlx CLI, which can be done with cargo:

```bash
cargo install --git https://github.com/launchbadge/sqlx sqlx-cli --no-default-features --features postgres,rustls
```

From there, you can create the database and perform all database migrations with one simple command:

```bash
sqlx database setup
```

Finally, if on Linux, you will need the OpenSSL library. On Debian-based systems, this involves the `pkg-config` and `libssl-dev` packages.

To enable labrinth to create a project, you need to add two things.

1. An entry in the `loaders` table.
2. An entry in the `loaders_project_types` table.

A minimal setup can be done from the command line with [psql](https://www.postgresql.org/docs/current/app-psql.html):

```bash
psql --host=localhost --port=5432 -U <username, default is labrinth> -W
```

The default password for the database is `labrinth`. Once you've connected, run

```sql
INSERT INTO loaders VALUES (0, 'placeholder_loader');
INSERT INTO loaders_project_types VALUES (0, 1); -- modloader id, supported type id
INSERT INTO categories VALUES (0, 'placeholder_category', 1); -- category id, category, project type id
```

This will initialize your database with a modloader called 'placeholder_loader', with id 0, and marked as supporting mods only. It will also create a category called 'placeholder_category' that is marked as supporting mods only
If you would like 'placeholder_loader' to be marked as supporting modpacks too, run

```sql
INSERT INTO loaders_project_types VALUES (0, 2); -- modloader id, supported type id
```

If you would like 'placeholder_category' to be marked as supporting modpacks too, run

```sql
INSERT INTO categories VALUES (0, 'placeholder_category', 2); -- modloader id, supported type id
```

You can find more example SQL statements for seeding the database in the `apps/labrinth/tests/files/dummy_data.sql` file.

The majority of configuration is done at runtime using [dotenvy](https://crates.io/crates/dotenvy) and the `.env` file. Each of the variables and what they do can be found in the dropdown below. Additionally, there are three command line options that can be used to specify to MeiliSearch what you want to do.

During development, you might notice that changes made directly to entities in the PostgreSQL database do not seem to take effect. This is often because the Redis cache still holds outdated data. To ensure your updates are reflected, clear the cache by e.g. running `redis-cli FLUSHALL`, which will force Labrinth to fetch the latest data from the database the next time it is needed.

<details>
<summary>.env variables & command line options</summary>

#### Basic configuration

`DEBUG`: Whether debugging tools should be enabled  
`RUST_LOG`: Specifies what information to log, from rust's [`env-logger`](https://github.com/env-logger-rs/env_logger); a reasonable default is `info,sqlx::query=warn`  
`SITE_URL`: The main URL to be used for CORS  
`CDN_URL`: The publicly accessible base URL for files uploaded to the CDN  
`MODERATION_DISCORD_WEBHOOK`: The URL for a Discord webhook where projects pending approval will be sent  
`CLOUDFLARE_INTEGRATION`: Whether labrinth should integrate with Cloudflare's spam protection  
`DATABASE_URL`: The URL for the PostgreSQL database  
`DATABASE_MIN_CONNECTIONS`: The minimum number of concurrent connections allowed to the database at the same time  
`DATABASE_MAX_CONNECTIONS`: The maximum number of concurrent connections allowed to the database at the same time  
`MEILISEARCH_ADDR`: The URL for the MeiliSearch instance used for search  
`MEILISEARCH_KEY`: The name that MeiliSearch is given  
`BIND_ADDR`: The bind address for the server. Supports both IPv4 and IPv6  
`MOCK_FILE_PATH`: The path used to store uploaded files; this has no default value and will panic if unspecified
`SMTP_USERNAME`: The username used to authenticate with the SMTP server
`SMTP_PASSWORD`: The password associated with the `SMTP_USERNAME` for SMTP authentication
`SMTP_HOST`: The hostname or IP address of the SMTP server
`SMTP_PORT`: The port number on which the SMTP server is listening (commonly 25, 465, or 587)
`SMTP_TLS`: The TLS mode to use for the SMTP connection, which can be one of the following: `none`, `opportunistic_start_tls`, `requires_start_tls`, `tls`

#### CDN options

`STORAGE_BACKEND`: Controls what storage backend is used. This can be either `local`, `backblaze`, or `s3`, but defaults to `local`

The Backblaze and S3 configuration options are fairly self-explanatory in name, so here's simply their names:  
`BACKBLAZE_KEY_ID`, `BACKBLAZE_KEY`, `BACKBLAZE_BUCKET_ID`  
`S3_ACCESS_TOKEN`, `S3_SECRET`, `S3_URL`, `S3_REGION`, `S3_BUCKET_NAME`

#### Search, OAuth, and miscellaneous options

`LOCAL_INDEX_INTERVAL`: The interval, in seconds, at which the local database is reindexed for searching. Defaults to `3600` seconds (1 hour).  
`VERSION_INDEX_INTERVAL`: The interval, in seconds, at which versions are reindexed for searching. Defaults to `1800` seconds (30 minutes).

The OAuth configuration options are fairly self-explanatory. For help setting up authentication, please contact us on [Discord].

`RATE_LIMIT_IGNORE_IPS`: An array of IPs that should have a lower rate limit factor. This can be useful for allowing the front-end to have a lower rate limit to prevent accidental timeouts.

#### Command line options

`--skip-first-index`: Skips indexing the local database on startup. This is useful to prevent doing unnecessary work when frequently restarting.  
`--reconfigure-indices`: Resets the MeiliSearch settings for the search indices and exits.  
`--reset-indices`: Resets the MeiliSearch indices and exits; this clears all previously indexed mods.

</details>

#### Ready to open a PR?

If you're prepared to contribute by submitting a pull request, ensure you have met the following criteria:

- `cargo fmt` has been run.
- `cargo clippy` has been run.
- `cargo check` has been run.
- `cargo sqlx prepare` has been run.

> Note: If you encounter issues with `sqlx` saying 'no queries found' after running `cargo sqlx prepare`, you may need to ensure the installed version of `sqlx-cli` matches the current version of `sqlx` used [in labrinth](https://github.com/modrinth/labrinth/blob/master/Cargo.toml).

[Discord]: https://discord.modrinth.com
[GitHub]: https://github.com/modrinth
[labrinth]: https://github.com/modrinth/labrinth
[Rust]: https://www.rust-lang.org/tools/install
