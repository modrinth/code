---
title: Theseus (Modrinth App)
description: Guide for contributing to Modrinth's desktop app
---

This project is part of our [monorepo](https://github.com/modrinth/code). You can find it in the `apps/app` directory.

[theseus] is the Tauri-based launcher that lets users conveniently play any mod or modpack on Modrinth. It uses the Rust-based Tauri as the backend and Vue.js as the frontend. To get started, install [pnpm], [Rust], and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your system. Then, run the following commands:

```bash
pnpm install
pnpm run app:dev
```

Once the commands finish, you'll be viewing a Tauri window with Nuxt.js hot reloading.

You can use `pnpm run lint` to find any eslint problems, and `pnpm run fix` to try automatically fixing those problems.

### Theseus Architecture

Theseus is split up into three parts:

- `apps/app-frontend`: The Vue.JS frontend for the app
- `packages/app-lib`: The library holding all the core logic for the desktop app
- `apps/app`: The Tauri-based Rust app. This primarily wraps around the library with some additional logic for Tauri.

The app's internal database is stored in SQLite. For production builds, this is found at <APPDIR>/app.db.

When running SQLX commands, be sure to set the `DATABASE_URL` environment variable to the path of the database.

You can edit the app's data directory using the `THESEUS_CONFIG_DIR` environment variable.

#### Ready to open a PR?

If you're prepared to contribute by submitting a pull request, ensure you have met the following criteria:

- Run `pnpm run fix` to address any fixable issues automatically.
- Run `cargo fmt` to format Rust-related code.
- Run `cargo clippy` to validate Rust-related code.
- Run `cargo sqlx prepare --package theseus` if you've changed any SQL code to validate statements.

[theseus]: https://github.com/modrinth/code/tree/main/apps/app
[Rust]: https://www.rust-lang.org/tools/install
[pnpm]: https://pnpm.io
