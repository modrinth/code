# Agent notes — Owyx launcher

Canonical product: **Owyx** = site (`owyxsite/`) + launcher (`apps/app` + `apps/app-frontend` + `packages/app-lib`), Modrinth-App-like shell without their brand/Hosting. Not a single-game-server product; plugin is out of scope (history in OwyxOld).

This repo is a **fork of modrinth/code** — same package licenses (GPL-3 for the app); no Modrinth trademarks. See root `README.md`, `LICENSE`, `COPYING.md`, `TRADEMARK.md`.

## Do

- Follow upstream `modrinth/code` for features/fixes; **keep the GitHub fork relationship**.
- Brand from **`brand/`** — `brand/DESIGN.md`, prefer `brand/v2/` (Sora/Onest/Unbounded, cyan `#00e5ff`, bg `#050508`). Owyx™ marks: `TRADEMARK.md`.
- Control-plane site: **`owyxsite/`** (Next frontend + Node API + deploy). Not Modrinth `apps/frontend` / `labrinth`.
- API: `https://api.owyx.site` + header `X-Owyx-Client-Key` (placeholder / settings only — **never commit real keys**). Contract: `owyxsite/LAUNCHER_SITE_CONTRACT.md`.
- User-facing accounts: **Owyx site login** and **Microsoft** (`packages/app-lib` auth); see `docs/ms-oauth.md`.
- Old stack archive: **https://github.com/ebluffy/OwyxOld** only (plugin history lives there — no plugin in this repo).

## Do not

- Ship Modrinth logos, wrench-in-labyrinth marks, green-as-primary, Hosting/Medal / Modrinth+ upsell chrome, or claim to be Modrinth.
- Re-add a `legacy/` folder into this repo.
- Mass-rename `@modrinth/*` packages (upstream merge hygiene) — document as a follow-up.
- Commit `.env`, real client keys, `*.exe`, `frpc.toml` secrets, or other secrets.
- Unfork / delete the upstream remote / rewrite history to hide origin.
- Touch `apps/frontend` / `apps/labrinth` for Owyx product work.
- Reuse **Owyx™** branding in forks (see `TRADEMARK.md` / `COPYING.md`).

## Pointers

| Topic | Where |
|-------|--------|
| Design tokens | `brand/DESIGN.md` |
| Logos / icons | `brand/v2/` |
| Brand / trademarks | `TRADEMARK.md`, `COPYING.md` |
| MS OAuth | `docs/ms-oauth.md` |
| Site ↔ launcher contract | `owyxsite/LAUNCHER_SITE_CONTRACT.md` |
| Owyx Servers client | `apps/app-frontend/src/helpers/owyx-api.ts` |
| Owyx / MS auth | `packages/app-lib/src/state/minecraft_auth.rs` (+ site JWT) |
| SemVer releases | `.cursor/rules/semver.mdc` — baseline `0.2.0`, bump MAJOR/MINOR/PATCH per https://semver.org/ |
| External agent skills / MCP | [ebluffy/agent-tool-catalog](https://github.com/ebluffy/agent-tool-catalog) · skill `.agents/skills/agent-tool-catalog/` |

Cloud task paste (local only): `promt.md` (gitignored).
