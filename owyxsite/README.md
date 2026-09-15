# Owyx stack

Hybrid migration target for the former Chiwawa site:

```
PostgreSQL 16  ← owyx/postgres/init.sql (+ migrations/001_chat.sql)
     ↑
Directus       ← CMS / admin UI (:8055, path /directus)
     ↑
Backend API    ← owyx/backend Express + Socket.io (:3001, paths /api/*)
     ↑
Next.js        ← owyx/frontend (:3000, rewrites /api → backend)
```

`chiwawasite/` remains in the repo as a behavior reference and is **not** required at runtime.

**Design:** the site follows the launcher-aligned design system in the root `DESIGN.md`
(dark space + cyan `#00e5ff`, DM Sans). Pre-redesign gold UI is snapshotted in `old.backup/`.
Launcher integration spec: `LAUNCHER_SITE_CONTRACT.md` (Owyx-account login, `GET /api/launcher/me`).

**Versions:** Site UI **0.1** (аккаунт, download, auth, cosmetics, admin catalog) ·
Launcher-facing API **v1.1** (`GET /api/launcher/v1/status` → `version 1.1.0`, `siteVersion 0.1.0`).
Catalog: `GET /api/launcher/v1/servers|packs|news` are live rows, not empty stubs.
Admin CRUD: `/api/admin/servers` + `/api/admin/packs`. See `LAUNCHER_SITE_CONTRACT.md`.

## Quick start (local, without Docker)

1. Run PostgreSQL 17 (Compose) or 16 (Cloud/apt is enough for local API work). Create roles `owyx`/`root` (`postgres/migrations/000_bootstrap_roles.sql`), apply `postgres/init.sql`, then migrations `001_chat.sql` … `007_servers_packs.sql`. Catalog tables created by `ensureCatalogSchema()` must be **owned by** `owyx_user`.
2. Copy env: `cp .env.example .env` — set `DB_*`, `JWT_SECRET`, `TURNSTILE_SKIP=true`.
   Turnstile on the frontend is opt-in: it renders only when `NEXT_PUBLIC_TURNSTILE_SITE_KEY` is set.
3. Backend:
   ```bash
   cd backend && npm install && npm run dev
   # http://127.0.0.1:3001/health
   ```
4. Frontend:
   ```bash
   cd frontend && npm install
   BACKEND_URL=http://127.0.0.1:3001 npm run dev
   # http://127.0.0.1:3000
   ```

## Docker Compose (one command)

```bash
cd owyxsite
cp .env.example .env          # set DB_PASSWORD, JWT_SECRET, SESSION_SECRET (+ optional SMTP)
docker compose up -d --build  # postgres + adminer + backend + frontend
```

| Service  | URL / Port                    | Notes |
|----------|-------------------------------|-------|
| frontend | http://localhost:3000         | Next.js site |
| backend  | http://localhost:3001/health  | Express API + Socket.io |
| adminer  | http://localhost:8080         | DB UI — System **PostgreSQL**, Server `postgres`, user/db from `.env` |
| postgres | localhost:5432                | Postgres **17** (`\restrict` in the dump needs 16.4+/17) |
| directus | http://localhost:8055         | optional: `docker compose --profile directus up -d` |

**Fresh DB bootstrap.** On first boot (empty `./postgres/data`) Postgres runs
`docker-entrypoint-initdb.d` in filename order: `000_bootstrap_roles` (create
`owyx`/`root` — the dump does `OWNER TO owyx|root`) → `init.sql` → `001_chat` →
`002_roles` → `003_cosmetics` (skins) → `004_nickname_cooldown` →
`005_email_change` → `006_news` → `007_servers_packs`. Re-init from
scratch: `docker compose down && rm -rf postgres/data`.

**Existing volume does not auto-apply new SQL.** `initdb.d` runs only on first
create. On a DB that already has data, apply the new file yourself:

```bash
docker compose exec -T postgres psql -U owyx_user -d owyx_db < postgres/migrations/007_servers_packs.sql
# or locally:
# psql -U owyx_user -d owyx_db -f postgres/migrations/007_servers_packs.sql
```

The backend also runs an idempotent `ensureCatalogSchema()` on startup (CREATE
TABLE IF NOT EXISTS + demo seed if empty), so friends-dev often works without
the manual step — still apply the file on production volumes so indexes/seed
match git.

Apply any older migration to a running DB the same way:

```bash
docker compose exec -T postgres psql -U owyx_user -d owyx_db < postgres/migrations/006_news.sql
```

Production: loopback binds + host nginx. See [`deploy/README.md`](./deploy/README.md)
(`bash deploy/vps-up.sh`; set `TURNSTILE_SECRET_KEY` first). `panel.owyx.site` stays with C³ CELERITY — do not
steal host `:443`. Apex `owyx.site` proxies the Next app (no Coming Soon).

## Launcher dev against local compose

```bash
# after docker compose is healthy
export OWYX_API_BASE_URL=http://127.0.0.1:3001
cd ../Launcher/apps/launcher
npm run check && npm run check:rust
npm run dev:tauri
```

The native window talks to Express on `:3001` (catalog, login, admin, skins).
Vite-only `npm run dev` still proxies `/proxy/owyx` to the same base.

Launcher overlay cap: **512 MB** (`MAX_PACK_BYTES`). Larger packs (Foreg ~2 GB)
need a streaming download — not this stack.

## Email

HTML templates (dark + cyan, inline CSS) live in
`backend/src/utils/emailTemplates.js` (verify / reset / password-changed /
welcome / nickname-changed). Without SMTP they are simulated in logs. Preview:

```bash
cd backend && node scripts/preview-emails.js   # writes email-previews/*.html
# or, as admin: GET /api/admin/email-preview/<key>
```

SMTP (Yandex) via `SMTP_HOST/PORT/USER/PASS` + `EMAIL_FROM` in `.env` — **never
commit the password**.

## Directus

Optional CMS: `docker compose --profile directus up -d`. See
[`directus/README.md`](./directus/README.md) for tables and role checklist.
