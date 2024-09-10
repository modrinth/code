# ![Modrinth App](/.github/assets/app_cover.png)

## Modrinth App

The Modrinth App is a desktop application for managing your Minecraft mods. It is built with [Tauri](https://tauri.app/) and [Vue](https://vuejs.org/).

If you're not a developer and you've stumbled upon this repository, you can download the latest release of the app from the [Modrinth website](https://modrinth.com/app).

## Development

### Pre-requisites

Before you begin, ensure you have the following installed on your machine:

- [Node.js](https://nodejs.org/en/)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri](https://v2.tauri.app/start/prerequisites/)

### Setup

Follow these steps to set up your development environment:

```bash
cargo install tauri-cli --git https://github.com/modrinth/tauri.git --rev c2b059b85370e1a7018faf3286d2cd8b8ce58a38
pnpm install
pnpm app:dev
```

You should now have a development build of the app running with hot-reloading enabled. Any changes you make to the code will automatically refresh the app.
