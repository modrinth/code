# Agent notes — Owyx (Modrinth code fork)

Canonical product: **Owyx launcher** = `apps/app` + `apps/app-frontend` + `packages/app-lib`, rebranded.

- **Do not** ship Modrinth logos / cover images / green brand primary.
- **Do** follow upstream `modrinth/code` for features/fixes; keep fork relationship.
- Design + logos: **`brand/`** (`brand/DESIGN.md`, prefer `brand/v2/`).
- Control-plane: `api.owyx.site` + `X-Owyx-Client-Key` (see cloud `promt.md` / site contract on OwyxOld).
- Old stack archive: `ebluffy/OwyxOld` only — **no `legacy/` folder in this repo**.
- Cloud task paste: `promt.md` (gitignored).

Tokens: `#050508` / `#00e5ff` (see `brand/DESIGN.md`).
