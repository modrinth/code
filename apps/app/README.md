# Owyx desktop app (Tauri)

Part of the **[ebluffy/Owyx](https://github.com/ebluffy/Owyx)** monorepo — a public fork/rebrand of [modrinth/code](https://github.com/modrinth/code).

Product name: **Owyx**. Source license: **GPL-3** (same as upstream). See root [`LICENSE`](../../LICENSE) and [`COPYING.md`](../../COPYING.md).

## Dev

From the monorepo root:

```bash
pnpm install
pnpm app:dev
```

Release bundles (CI): `.github/workflows/owyx-github-release.yml` using `tauri-owyx-release.conf.json` (no Modrinth updater CDN / code signing).
