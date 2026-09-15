# Agent notes — Owyx launcher

Canonical product: **Owyx** = `apps/app` + `apps/app-frontend` + `packages/app-lib`, rebranded from the Modrinth App shell. Control-plane site: **`owyxsite/`**.

This repo is a **fork of modrinth/code** — same package licenses (GPL-3 for the app); no Modrinth trademarks. See root `README.md`, `LICENSE`, `COPYING.md`.

## Do

- Follow upstream `modrinth/code` for features/fixes; **keep the GitHub fork relationship**.
- Brand from **`brand/`** — `brand/DESIGN.md`, prefer `brand/v2/` (Sora, cyan `#00e5ff`, bg `#050508`).
- Control-plane site: **`owyxsite/`** (Next frontend + Node API + deploy). Not Modrinth `apps/frontend` / `labrinth`.
- API: `https://api.owyx.site` + header `X-Owyx-Client-Key` (placeholder / settings only — **never commit real keys**). Contract: `owyxsite/LAUNCHER_SITE_CONTRACT.md`.
- Offline nickname + Microsoft login both live in `packages/app-lib` auth; see `docs/ms-oauth.md`.
- Old stack archive: **https://github.com/ebluffy/OwyxOld** only (plugin history lives there — no plugin in this repo).

## Do not

- Ship Modrinth logos, wrench-in-labyrinth marks, green-as-primary, Hosting/Medal / Modrinth+ upsell chrome, or claim to be Modrinth.
- Re-add a `legacy/` folder into this repo.
- Mass-rename `@modrinth/*` packages (upstream merge hygiene) — document as a follow-up.
- Commit `.env`, real client keys, `*.exe`, `frpc.toml` secrets, or other secrets.
- Unfork / delete the upstream remote / rewrite history to hide origin.
- Touch `apps/frontend` / `apps/labrinth` for Owyx product work.

## Pointers

| Topic | Where |
|-------|--------|
| Design tokens | `brand/DESIGN.md` |
| Logos / icons | `brand/v2/` |
| MS OAuth | `docs/ms-oauth.md` |
| Site ↔ launcher contract | `owyxsite/LAUNCHER_SITE_CONTRACT.md` |
| Owyx Servers client | `apps/app-frontend/src/helpers/owyx-api.ts` |
| Offline accounts | `Credentials::create_offline` in `packages/app-lib/src/state/minecraft_auth.rs` |
| SemVer releases | `.cursor/rules/semver.mdc` — baseline `0.2.0`, bump MAJOR/MINOR/PATCH per https://semver.org/ |

Cloud task paste (local only): `promt.md` (gitignored).
