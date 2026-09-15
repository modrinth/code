# old.backup — legacy site snapshots

Snapshot of the **pre-redesign** Owyx site frontend, taken during the
launcher-aligned redesign (2026-08-06, branch `site/launcher-aligned-redesign`).

Kept in git on purpose (not gitignored) so any legacy gold-UI screen or a
community section that was turned into a "coming soon" stub can be restored.

## What's here

| Path | What it is |
|------|------------|
| `frontend-app/` | Full copy of `owyxsite/frontend/app/**` **before** the redesign (gold Minecraft UI): landing, auth pages, profile, admin, and the community pages `forum/`, `chat/`, `shop/`, `tokens/`, `online/`. |
| `frontend-components/` | Full copy of `owyxsite/frontend/components/**` before redesign: gold `Header`, `Footer`, `HeroSection`, `DiscordSection`, `ParticlesBackground`, `server/ServerStatus`, `server/CopyIPButton`. |

## What moved / changed in the new site

- **Design tokens & fonts** rewritten in `frontend/app/globals.css` + `frontend/app/layout.tsx`
  to the launcher palette (`#050508` bg, cyan `#00e5ff` accent, DM Sans) — no more gold `#FFAA00`.
- **Landing / auth / profile / admin** rewritten in the new design system (API calls unchanged).
- **New page** `frontend/app/servers/` (private-servers story + launcher CTA).
- **Community sections** `forum`, `chat`, `shop`, `tokens`, `online` are now honest
  "coming soon" stubs in the new skin and removed from the primary nav. Their original
  implementations live here under `frontend-app/<section>/`.

## Backend / API

No backend routes were deleted. The full auth / applications / admin / profile /
game-token API is unchanged (the plugin depends on it). A small additive
`GET /api/launcher/me` alias was added for the launcher (see
`owyxsite/LAUNCHER_SITE_CONTRACT.md`); nothing was removed.

## How to restore a legacy screen

1. Copy the file back, e.g.
   `cp old.backup/frontend-app/forum/page.tsx frontend/app/forum/page.tsx`.
2. Restyle it to the new tokens (see `globals.css`) or accept the legacy gold look.
3. Re-add its link to `frontend/components/layout/Header.tsx` nav if desired.
