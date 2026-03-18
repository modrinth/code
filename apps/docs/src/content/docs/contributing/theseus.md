---
title: Theseus (Modrinth App)
description: Guide for contributing to Modrinth's desktop app
sidebar:
  order: 3
---

[Theseus] is the Tauri-based launcher that lets users conveniently play any mod or modpack on Modrinth. It uses the Rust-based Tauri as the backend and Vue.js as the frontend.

## Setup

### 1. Install prerequisites

- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### 2. Install dependencies & set up .env

- Clone [`https://github.com/modrinth/code`](https://github.com/modrinth/code) and run `pnpm install` in the workspace root folder.
- In `packages/app-lib` you should be able to see `.env.prod`, `.env.staging` — for basic app work, it's recommended to use `.env.prod`. Copy the relevant file into a new `.env` file within the `packages/app-lib` folder.

### 3. Run the app

- Run `pnpm app:dev` in the workspace root folder.
- If you encounter an "Unable to initialise GTK" error on Linux, try running `pnpm run dev` within the `apps/app` folder rather than the workspace root command. If this doesn't work, use a VM with Windows and repeat these steps.
- If you get issues with being unable to find a display, and you're running the dev env inside a container (e.g. an Arch dev container on a non-Arch host), run `xhost +local:` to allow it to bind to an X11/Xwayland socket.

## Theseus Architecture

Theseus is split up into three parts:

- `apps/app-frontend`: The Vue.JS frontend for the app
- `packages/app-lib`: The library holding all the core logic for the desktop app
- `apps/app`: The Tauri-based Rust app. This primarily wraps around the library with some additional logic for Tauri.

The app's internal database is stored in SQLite. For production builds, this is found at <APPDIR>/app.db.

When running SQLX commands, be sure to set the `DATABASE_URL` environment variable to the path of the database.

You can edit the app's data directory using the `THESEUS_CONFIG_DIR` environment variable.

## Ready to open a PR?

If you're prepared to contribute by submitting a pull request, ensure you have met the following criteria:

- Run `pnpm prepr:frontend` to address any fixable issues automatically.
- Run `cargo fmt` to format Rust-related code.
- Run `cargo clippy` to validate Rust-related code.
- Run `cargo sqlx prepare --package theseus` if you've changed any SQL code to validate statements.

[theseus]: https://github.com/modrinth/code/tree/main/apps/app
[Rust]: https://www.rust-lang.org/tools/install
[pnpm]: https://pnpm.io
