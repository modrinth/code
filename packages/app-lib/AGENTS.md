# packages/app-lib - Modrinth App Library (`theseus`)

Rust crate holding the core logic of the Modrinth App: instances, installs, launching, shared instances, and the SQLite state. `apps/app` (crate `theseus_gui`) wraps it as Tauri commands in `apps/app/src/api/`, and `apps/app-frontend` calls those commands with `invoke`.

## Checks

- Use `cargo clippy -p theseus -p theseus_gui --all-targets` instead of `cargo check`. CI runs `cargo clippy --all-targets --locked`.
- Format with `cargo fmt --all`. CI runs `cargo fmt --all --check`.
- If a change adds or renames frontend strings in the app, also run `pnpm intl:extract` in `apps/app-frontend`.

## SQLx

Queries are checked at compile time against the offline data in `packages/app-lib/.sqlx`. If `DATABASE_URL` isn't set in `packages/app-lib/.env`, build with `SQLX_OFFLINE=true`.

After changing a query or adding a migration, regenerate the offline data against a scratch database:

```bash
cd packages/app-lib
export DATABASE_URL="sqlite://$TMPDIR/theseus-sqlx.db"
cargo sqlx database setup
cargo sqlx prepare
```

Add new migrations to `migrations/`. Never edit an existing migration, since users' databases have already applied it.

## Build

`build.rs` builds the Java helper jars in `java/` with Gradle (`java/gradlew`) on the first build and whenever `java/` changes. This needs a JDK, and the first build is slow.

## Logging

Use `tracing`. In dev builds, the default filter is `theseus=info,theseus_gui=info,webview=trace`. Override it with `RUST_LOG`:

```bash
RUST_LOG=theseus=debug pnpm app:dev
```
