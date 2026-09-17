# Launcher ↔ Site contract

How the **Owyx launcher** talks to the **Owyx site** API. Source of truth for the
launcher/site boundary. Keep in sync with `backend/src/routes/launcher.js`.

**Versions:** launcher-facing **API surface `1.3.0`** (`GET /api/launcher/v1/status.version`),
**site UI `0.1.0`** (`.siteVersion`). Additive: friends (`/api/friends`), catalog ACL
(`access_mode` + `catalog_acl`), optional Bearer on catalog lists, social settings
(`GET|PATCH /api/friends/settings`), CustomSkinLoader public API (`/api/csl`).

**Updated:** 2026-09-17 — friends privacy settings, CSL skin API, presence contract unchanged (~90s).

---

## Product model (important)

Playing is **open**:

- **Guest / offline** — pick a nick in the launcher and Play. No account.
- **Owyx account** — register on the site, sign in inside the launcher with the
  same account, play. Later: skin, cape, profile perks.
- **Microsoft** — separate licensed profile (real OAuth is future work; honest
  stub for now). Never breaks offline or Owyx.

**Applications («заявки») are no longer a whitelist gate.** A registered account
that is **active and not banned** may play. A **ban** blocks everything.

---

## Base URL

The launcher targets one API base:

| Env | Base URL |
|-----|----------|
| dev | `http://127.0.0.1:3001` |
| prod | `https://api.owyx.site` |

Configured in the launcher via `OWYX_API_BASE_URL` (see `Launcher/.env.example`)
or the launcher config. All paths below are relative to the base.

**Website** is `https://owyx.site` (browser uses same-origin `/api` via nginx → backend).
Do not point the launcher at `owyx.site` for catalog downloads in prod — use `api.owyx.site`.

### Client key (not a public API)

Direct requests to **`api.owyx.site`** must send:

```http
X-Owyx-Client-Key: <LAUNCHER_CLIENT_KEY>
```

The key is configured as `LAUNCHER_CLIENT_KEY` on the site and `OWYX_CLIENT_KEY`
(or `%USERPROFILE%/owyx/client_key`) in the launcher. Missing/wrong key → `401 unauthorized_client`.
`/health` stays open. Browser traffic via `https://owyx.site/api` does **not** need the key
(`Host: owyx.site`).

---

## Auth flow (Owyx account in the launcher)

1. `POST /api/auth/login` with `{ "login", "password", "remember": true }`
   (`email` still accepted as an alias for `login`).
   - Browser (Host: `owyx.site`): may require Cloudflare Turnstile (`turnstileToken`).
   - **Launcher** (Host: `api.owyx.site` **and** valid `X-Owyx-Client-Key`):
     Turnstile is **skipped**. Browser Host (`owyx.site`) always requires captcha
     even if the client key header is present.
   - `200` → `{ success: true, token, user }`. `token` is a JWT.
   - `401`/`403` → `{ error }` (wrong credentials / inactive). Show a human message.
   - `401 unauthorized_client` → missing/invalid client key (configure in Owyx Servers settings).
2. Store the JWT securely (OS app-data, never plaintext in the UI).
3. `GET /api/launcher/me` with `Authorization: Bearer <token>` → profile + access.
   Session rows store a SHA-256 of the JWT (legacy base64 hashes are migrated on
   use). Logout deletes the matching session hash.
4. `serverAccess === true` → the account may play. `false` → show `accessReason`.
5. Logout: drop the stored token (optionally `POST /api/auth/logout`).

**Parallel sessions:** login does **not** invalidate other clients. Site and
launcher can stay signed in together. The server keeps up to ~10 active sessions
per user (oldest dropped). A `401`/`403` from `/me` means the JWT is gone —
launcher maps that to `no_session` and asks the user to sign in again.

Offline profiles never call the API; a nick is enough to Play.

**Account editing** (email, nick, password, Discord) is on the **website** ЛК
(`/profile`), not yet mirrored as launcher settings APIs. Launcher signs in and
reads `/me` + catalog.

**Friends / Social** in the desktop UI use the **Owyx control-plane** friends API
(`GET/POST /api/friends…` on `api.owyx.site`), not the upstream Modrinth
`plugin:friends` / Labrinth identity. The launcher helpers are
`apps/app-frontend/src/helpers/owyx-friends.ts` and `owyx-presence.ts`; the
sidebar UI is `FriendsList.vue`. Auth is Bearer JWT (site session) plus
`X-Owyx-Client-Key` on the API host. Presence heartbeats (`POST /api/friends/presence`)
use a server-side TTL of about **90 seconds** — after that the friend shows offline.
Catalog seed names like `owyx-friends` are unrelated demo servers.

---

## Endpoints

### `POST /api/auth/login`
Request: `{ "login": string, "password": string }` (`email` accepted as alias)
Response `200`:
```json
{ "success": true, "token": "<jwt>", "user": { "id": 1, "nickname": "Steve", "role": "user", ... } }
```

### `GET /api/launcher/me`  (auth: `Bearer <jwt>`)
Also available as `GET /api/launcher/v1/me`.
```json
{
  "user": {
    "id": 1,
    "nickname": "Steve",
    "email": "steve@owyx.local",
    "role": "user",
    "trustLevel": 0,
    "banned": false,
    "emailVerified": false,
    "registeredAt": "2026-08-06T01:13:43.477Z"
  },
  "serverAccess": true,
  "accessReason": "ok",          // "ok" | "banned" | "inactive"
  "cosmetics": {
    "skinUrl": "https://api.owyx.site/uploads/skins/skin-1-....png",
    "skinModel": "classic",      // "classic" | "slim"
    "capeUrl": null
  },
  "skinUrl": "https://api.owyx.site/uploads/skins/skin-1-....png",
  "application": null            // DEPRECATED: always null, removed next release
}
```
- `serverAccess` = account is active **and** not banned.
- `skinUrl` is absolute, built from the request host, so it is downloadable by
  whatever base the launcher used.

### `GET /api/launcher/v1/status`  (public)
```json
{
  "api": "owyx-launcher",
  "version": "1.2.0",
  "siteVersion": "0.1.0",
  "serverAccessModel": "open",
  "auth": { "login": "/api/auth/login", "me": "/api/launcher/me" },
  "modules": ["servers", "packs", "news", "cosmetics", "adminCatalog"]
}
```

### Friends (`/api/friends`, auth: Bearer + client key on api host)

- `GET /api/friends` → `{ friends: [{ id, userId, nickname, avatarUrl, status, incoming, presence, instanceName, presenceUpdatedAt, ... }], incomingCount }`
- `GET /api/friends/search?q=` → `{ users: [...] }`
- `POST /api/friends/request` `{ nickname }` → create pending (or auto-accept reciprocal); respects target `allowFriendRequests`
- `POST /api/friends/:id/accept` · `POST /api/friends/:id/decline` · `DELETE /api/friends/:id`
- `POST /api/friends/presence` `{ status: "online"|"playing"|"offline", instanceName? }` — launcher heartbeat; presence rows older than ~90s are treated as offline.
- `GET /api/friends/settings` → `{ settings: { allowFriendRequests: boolean } }`
- `PATCH /api/friends/settings` `{ allowFriendRequests: boolean }` → upsert privacy

### CustomSkinLoader / public skins (`/api/csl`, no client key)

- `GET /api/csl/skins/{nickname}.png` — Legacy skin PNG (or redirect to uploaded asset)
- `GET /api/csl/{nickname}.json` — CustomSkinAPI profile for CSL
- Prefer `https://owyx.site/api/csl/` from in-game clients. See `docs/owyx-skins-in-world.md`.

### Catalog ACL

- `packs.access_mode` / `servers.access_mode`: `open` | `whitelist` | `blacklist`
- Junction `catalog_acl (resource_type, resource_id, user_id, effect)` with `allow`/`deny`
- `GET /api/launcher/v1/servers` and `/packs` accept **optional** Bearer JWT:
  - guest → only `open`
  - signed-in → whitelist/blacklist applied
- Admin: `GET|PUT /api/admin/packs|servers/:id/acl` with `{ accessMode, nicknames: string[] }`

### `GET /api/launcher/v1/cosmetics`  (auth)
`{ "skinUrl", "skinModel", "capeUrl", "updatedAt" }`.

### Catalog (site owns it; launcher is the client)

A **server** is where to connect (`address` + `port`). A **pack** is what to
download. One pack may back several servers. Players never receive SFTP
credentials — public GET returns only HTTP(S) download/manifest URLs.

Source types on a pack: `http_zip` | `http_manifest` | `google_drive` | `mrpack`
| `sftp` | `local_ingest`. Friends-MVP download path is **`http_zip` /
`http_manifest`**. Other types are stored + admin-editable; `sftp` is
admin-warehouse only (`downloadAvailable: false` in public GET). `mrpack` is
schema-only for now: public GET sets `downloadAvailable: false` and
`ingest: "planned"` even when a URL is stored. Drive without a direct URL is
not a fake download.

Dev seed (fresh DB / `ensureCatalogSchema`): pack `demo-vanilla` (tiny fixture
zip at `/fixtures/packs/demo-vanilla.zip`) and servers `owyx-demo` /
`owyx-friends`.

`GET /api/launcher/v1/servers` — published Owyx + community servers:
```json
{
  "servers": [
    {
      "id": "owyx-demo",
      "name": "Owyx — демо",
      "iconUrl": null,
      "address": "45.131.186.146:1488",
      "port": 25565,
      "kind": "owyx",
      "packId": "demo-vanilla",
      "minecraft": "1.21.1",
      "loader": "vanilla",
      "requiresAccount": false,
      "status": { "online": null, "players": null, "max": null },
      "pack": {
        "id": "demo-vanilla",
        "name": "Owyx Demo Vanilla 1.21.1",
        "minecraft": "1.21.1",
        "loader": "vanilla",
        "iconUrl": null,
        "description": "",
        "sourceType": "http_zip",
        "downloadUrl": "http://127.0.0.1:3001/fixtures/packs/demo-vanilla.zip",
        "sha256": "b5b345b133be7e851e47ef6f0fd83c61be3abc56dfc8cacd0c6fc65790d2eadb",
        "manifestUrl": null
      }
    }
  ]
}
```
Bearer is optional. `requiresAccount` is a Play gate in the launcher, not a
list filter. `status.online` is reserved (always `null` until ping exists).

`GET /api/launcher/v1/packs` — published builds (safe player fields only):
```json
{
  "packs": [
    {
      "id": "demo-vanilla",
      "name": "Owyx Demo Vanilla 1.21.1",
      "minecraft": "1.21.1",
      "loader": "vanilla",
      "sourceType": "http_zip",
      "downloadUrl": "http://127.0.0.1:3001/fixtures/packs/demo-vanilla.zip",
      "sha256": "<64 hex if set>",
      "manifestUrl": null,
      "description": ""
    }
  ]
}
```
`GET /api/launcher/v1/packs/:id` → `{ "pack": { … } }` (`404` if unpublished).
`GET /api/launcher/v1/packs/:id/manifest` → `{ id, name, minecraft, loader,
sourceType, files: [{ path, url, sha256, size }] }`. SFTP packs have empty
`files`.

`GET /api/launcher/v1/news` — same published `news` table as the site
(`GET /api/news`). `{ "news": [{ id, title, tag, summary, publishedAt, updatedAt }] }`.

### Admin catalog (JWT + role `admin`)

Same JWT as site/launcher login. Do not duplicate business rules in the
launcher — write through these routes.

- `GET/POST /api/admin/packs` · `GET/PUT/DELETE /api/admin/packs/:id`
- `POST /api/admin/packs/:id/ingest` — multipart field `archive` (`.zip` or
  `.mrpack`, ≤50 MB) → `/uploads/packs/{id}.zip` or `{id}.mrpack`. `.zip`
  becomes `http_zip` + sha256. `.mrpack` is stored but remains
  `downloadAvailable: false` until a real mrpack parser exists. Deleting a
  locally ingested pack also unlinks the file.
- `GET/POST /api/admin/servers` · `GET/PUT/DELETE /api/admin/servers/:id`

Create pack body (camelCase): `{ id?, name, minecraft, loader, iconUrl?,
description?, sourceType, source: { type, config }, published? }`.

`source.config` by type:

| type | config (admin) | public GET |
|------|----------------|------------|
| `http_zip` / `local_ingest` | `{ url, sha256? }` | `downloadUrl` + `sha256` |
| `http_manifest` | `{ manifestUrl, files? }` | `manifestUrl` |
| `google_drive` | `{ url, directDownloadUrl?, note? }` | `downloadUrl` only if direct |
| `mrpack` | `{ url }` | `downloadAvailable: false`, `ingest: "planned"` |
| `sftp` | `{ host, port, user, path, password? }` | `downloadAvailable: false` only |

Admin GET of an SFTP pack returns `source.config.hasPassword` and **never**
the password. Applications/whitelist stay gone. Ban still blocks `/me`.

---

## Cosmetics (skins)

- Site profile → **Внешний вид**: upload a PNG skin (`64×64` or legacy `64×32`),
  choose model (`classic`/`slim`). Cape is reserved (soon).
- Storage: file under `backend/uploads/skins/`, meta in `users.skin_url` /
  `users.skin_model` (migration `postgres/migrations/003_cosmetics.sql`).
- API (site, `Bearer` JWT):
  - `GET /api/profile/skin` → `{ skin_url, skin_model, cape_url, updated_at }`
  - `PUT /api/profile/skin` (multipart `skin` + optional `model`) — PNG only,
    validated with sharp, `512KB` cap, re-encoded to a clean PNG.
  - `DELETE /api/profile/skin` → back to default.
- Launcher: reads `cosmetics.skinUrl` / `skinModel` from `/api/launcher/me`,
  downloads the PNG to `%USERPROFILE%/owyx/skins/{nick}.png` (and optionally
  `instances/{id}/game/owyx/skin.png`). Full in-game apply (player-data /
  CustomSkinLoader) is still later; the file-on-disk step is wired.

---

## Community server auth (target flow)

The direction is **launcher-driven session**, not a copy-pasted token:

1. Player signs into the launcher with their **Owyx account** (`/api/auth/login`).
2. The launcher holds a session (JWT) and can prove identity to a server.
3. A **community server plugin** (the server owner's) validates that proof with
   the site — fast login / access without the player typing anything in chat.
4. Owner servers and "community" servers (owners who connected their server)
   appear in `/api/launcher/v1/servers`; `requiresAccount` decides offline-vs-Owyx.

The exact server↔site validation endpoint (e.g. a signed session-check for the
plugin) is **future work**; this section fixes the intent so the plugin and
launcher can be built against a stable idea.

### Legacy: game token (`/auth <token>` in chat) — **deprecated**

- `POST /api/auth/generate-game-token`, `verify-game-token`, `create-game-session`,
  `check-game-session` still exist for the **transitional** plugin and are not
  removed, but are **not promoted** in the site UI (the profile "game token" tab
  was removed). New servers should use the launcher-session flow above.
- `GET /api/plugin/server-access?nickname=...` and other
  `/plugin/*` **write/session** routes accept a **long-term API token only**
  (`authenticateLongTermApiTokenOnly`). A website session JWT is not enough,
  even for an admin. `hasAccess: true` for an **active, non-banned** account.
  Banned → `false`. The Express mount is `/api` + `/plugin/...` (not
  `/api/settings/plugin/...`).
- Whitelisting, if a server wants it, lives in the **plugin config**, never via
  site applications (applications are gone).

---

## News API

Home page + admin-managed updates.

- `GET /api/news?limit=6` (public) → published items, newest first:
  ```json
  { "news": [ { "id": 1, "title": "…", "tag": "Лаунчер", "summary": "…", "published": true, "created_at": "…" } ] }
  ```
- Admin (`Bearer` JWT, role `admin`):
  - `GET /api/admin/news` — all items incl. unpublished.
  - `POST /api/admin/news` `{ title, tag?, summary?, published? }` → `201 { news }`.
  - `PUT /api/admin/news/:id` `{ title?, tag?, summary?, published? }` → `{ news }`.
  - `DELETE /api/admin/news/:id` → `{ success: true }`.
- Storage: table `news` (migration `postgres/migrations/006_news.sql`), seeded with
  a couple of items on a fresh DB.

---

## Email change

Confirm a NEW address with a 6-digit code sent to that new address (`Bearer` JWT):

- `POST /api/profile/email/request` `{ email }` → sends code to the new email.
  60s cooldown. When SMTP is not configured and `NODE_ENV !== production`, the
  response includes `devCode` for local testing. Production never echoes the
  code. Errors: `400` bad/existing email, `409` taken, `429` too soon.
- `POST /api/profile/email/confirm` `{ code }` → applies the change, sets
  `is_email_verified = true`. Errors: `400` no request / expired / wrong code.
- Code lives 15 minutes (columns on `users`, migration `005_email_change.sql`).

Nickname change stays at **≤ 1 / 30 days** (`PUT /api/profile/nickname`,
migration `004_nickname_cooldown.sql`).

---

## Versioning

- Stable surface under `/api/launcher/v1/*`. `GET /v1/status.version` is the
  contract version (`1.1.0`). Additive changes bump the minor; breaking changes
  add `/v2/`.

## Launcher client (apps/app-frontend)

- Catalog fetch: `GET /api/launcher/v1/servers` (primary), with legacy fallbacks
  `/api/launcher/servers` and `/v1/launcher/catalog`.
- Maps contract fields: `name`, `address` (+ `port`), `minecraft`, `loader`,
  `iconUrl`, nested `pack.downloadUrl` → `packUrl`, `requiresAccount`.
- Site account: `POST /api/auth/login` + `GET /api/launcher/me` (JWT in app storage).
- Demo seed is **opt-in** (`owyx.demoServers`); default off.
- Security gates: CSP `api.owyx.site`, https-only pack/icon (reject `//`),
  http API base loopback-only, client key not sent to localhost fallback.
