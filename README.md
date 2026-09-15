# Owyx

**Owyx** is a desktop Minecraft launcher for private / friends servers: dark UI, cyan brand, curated Owyx Servers, and offline nickname login — with Microsoft login kept for licensed play.

This repository is a **public fork of [modrinth/code](https://github.com/modrinth/code)** so we can merge upstream fixes. **Product branding is Owyx**, not Modrinth.

| | |
|--|--|
| Website | https://owyx.site |
| Control plane | https://api.owyx.site |
| Brand kit | [`brand/`](./brand/) · [`brand/DESIGN.md`](./brand/DESIGN.md) · v2 logos in `brand/v2/` |
| Tokens | bg `#050508` · accent `#00e5ff` · font **Sora** (fallback Space Grotesk / Outfit / Geist) |

Old private monorepo (site / previous launcher / plugin archive only): **https://github.com/ebluffy/OwyxOld** — do **not** re-add a `legacy/` tree here.

## Layout

| Path | What |
|------|------|
| `apps/app` | Tauri shell → **Owyx** binary (`owyx://` deep links) |
| `apps/app-frontend` | Launcher UI (Vue) |
| `packages/app-lib` | Launcher core / Theseus (Rust) |
| `brand/` | Logos, hero art, design tokens |
| `owyxsite/` | Owyx control-plane (site + API + deploy) — our code, not upstream |
| `docs/ms-oauth.md` | Microsoft OAuth wiring for owners |
| `apps/frontend`, `apps/labrinth`, … | Upstream packages kept for sync |

## Features (launcher)

- **Owyx chrome** — logos, window title, Discord RPC, cyan theme
- **Owyx Servers** — catalog from `api.owyx.site` (`X-Owyx-Client-Key`; local settings + demo seed fallback)
- **Offline nickname** — deterministic offline UUID for offline-mode servers (Microsoft login unchanged)
- **Microsoft login** — public Minecraft client id; see `docs/ms-oauth.md`

`api.modrinth.com` may still be used as a **content catalog API host**. That is not product trademark UI.

## Dev

```bash
pnpm install
pnpm app:dev
```

Optional env (never commit secrets):

```bash
export OWYX_API_BASE_URL=http://127.0.0.1:3001
# VITE_OWYX_CLIENT_KEY=…   # frontend build-time placeholder only
```

Site / API (see `owyxsite/README.md` and `.env.example`):

```bash
cd owyxsite && docker compose up   # or documented local scripts
```

## Upstream sync

Keep the GitHub **fork relationship** to `modrinth/code`. Do not mass-rename `@modrinth/*` packages in drive-by PRs (follow-up when merge cost is understood).

```bash
git remote add upstream https://github.com/modrinth/code.git   # if missing
git fetch upstream
git merge upstream/main   # resolve brand / Owyx feature conflicts carefully
```

## License

See `LICENSE` / `COPYING.md` (AGPL and upstream attribution). No Modrinth trademark use in Owyx product UI.
