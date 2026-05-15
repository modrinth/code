# Icarus Launcher

Icarus Launcher is a high-performance desktop application for managing your Minecraft mods and instances. A powerful fork of the Icarus Launcher, built with [Tauri](https://tauri.app/) and [Vue](https://vuejs.org/), designed for speed and enhanced privacy.

## ✨ Unique Features

- **🔄 Cloud Sync**: Synchronize your instances, settings, and profiles across multiple devices seamlessly.
- **🎮 Offline Support**: Play your favorite instances without an internet connection, perfect for on-the-go gaming.
- **👑 Taxphobia Integration**: Native support for the Taxphobia ecosystem and branding.
- **🚀 Performance Focused**: Built on Rust and Vue for the snappiest experience possible.

If you're looking for the latest version, check out the [Releases](https://github.com/fraa2a/Icarus-Launcher/releases) page.

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
pnpm install
pnpm app:dev
```

You should now have a development build of the app running with hot-reloading enabled. Any changes you make to the code will automatically refresh the app.

### TaxPhobia news environment

To keep the API key out of source code, configure:

- `TAXPHOBIA_API_KEY` (required)
- `TAXPHOBIA_API_URL` (optional, defaults to `https://taxphobia.top/api`)

