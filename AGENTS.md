# Agent notes — Owyx (Modrinth code fork)

Canonical product: **Owyx launcher** = `apps/app` + `apps/app-frontend` + `packages/app-lib`, rebranded.

- **Do not** ship Modrinth logos / cover images / green brand primary.
- **Do** follow upstream `modrinth/code` for features/fixes; keep fork relationship.
- Design + logos: **`brand/`** (`brand/DESIGN.md`, prefer `brand/v2/`).
- Control-plane site: **`owyxsite/`** (Next frontend + Node API + deploy). Not Modrinth `apps/frontend` / `labrinth`.
- API: `api.owyx.site` + `X-Owyx-Client-Key` (see `owyxsite/LAUNCHER_SITE_CONTRACT.md`).
- Old stack archive: `ebluffy/OwyxOld` (plugin history lives there only — no plugin in this repo).
- Cloud task paste: `promt.md` (gitignored).

Tokens: `#050508` / `#00e5ff` (see `brand/DESIGN.md`).
