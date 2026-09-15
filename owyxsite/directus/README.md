# Directus on Owyx

Directus connects to the **same PostgreSQL** database as the custom Express API (`owyx_db`).
It is used as an admin/CMS UI for content and for future collections — **not** as the primary auth for the Minecraft plugin.

## First boot

1. Start the stack: `cd owyx && docker compose up -d`
2. Open Directus: `http://localhost:8055` (or behind `https://owyx.site` if proxied)
3. Log in with `ADMIN_EMAIL` / `ADMIN_PASSWORD` from `.env`

## Introspect existing tables

Existing app tables come from `postgres/init.sql` (`users`, `applications`, `forum_*`, etc.).
Directus will create its own system tables (`directus_*`) alongside them.

To manage existing tables in Directus:

1. **Settings → Data Model → Create Collection**
2. Choose **“Import existing table” / “Existing”** (wording depends on Directus version)
3. Pick tables you want to expose (start with: `server_settings`, `forum_categories`, `forum_topics`, `applications`)
4. For each collection, configure fields visibility and relationships

Do **not** let Directus own auth for game login — keep JWT sessions in Express (`/api/auth/*`).

## Role checklist

Create / map these Directus roles to match the site:

| Role | Purpose | Suggested access |
|------|---------|------------------|
| **Administrator** | Full Directus + all collections | Full CRUD |
| **Moderator** | Content + applications moderation | Read users; CRUD applications, forum_*, admin_logs (read) |
| **User** | Authenticated site users (optional Directus login later) | Read public content; create own forum posts |
| **Public** | Anonymous | Read `server_settings` public keys, forum categories (if published) |

Also keep the Express roles (`users.role`: `user` / `moderator` / `admin`) for API authorization — they are independent of Directus roles until you sync them intentionally.

## Recommended collections to expose first

- `server_settings` — editable from Directus *or* Express `/api/admin/settings`
- `forum_categories`, `forum_topics`, `forum_posts`
- `email_templates`
- `applications` (read-only in Directus if moderation stays in Express admin UI)

## Env

See `owyx/.env.example` (`DIRECTUS_KEY`, `DIRECTUS_SECRET`, `ADMIN_*`, `PUBLIC_URL`).
