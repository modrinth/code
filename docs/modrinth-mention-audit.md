# Modrinth mention audit (Owyx fork)

Scanned after PR #1 merge. Goal: no **Modrinth product trademarks** in the shipping Owyx launcher; keep **GPL attribution** and factual API references.

## Must keep (license / fork honesty)

| Kind | Examples |
|------|----------|
| License texts | Root `LICENSE` (GPL-3), package `LICENSE` / `COPYING.md` trademark bans from Rinth |
| Fork notice | Root `README.md`, GitHub About |
| Package names | `@modrinth/*` workspace packages (deferred rename for upstream merges) |
| Content API host | `api.modrinth.com` as catalog CDN (not product UI) |

## Fixed in this pass (shipping launcher)

- Welcome screen icon + EN/RU “Welcome to Owyx”
- Removed `modrinth_app.svg`, `sad-modrinth-bot.webp`, `modrinth-social-icon.png`
- NSIS old-install message → Owyx
- Linux `mainBinaryName` → `Owyx`
- Minecraft services User-Agent → Owyx
- Log compaction strings → Owyx
- Disabled Modrinth `updates.json` / updater endpoints for Owyx builds
- Maze placeholder → local `owyx-hero-bg.jpg`
- `tauri-owyx-release.conf.json` + `.github/workflows/owyx-github-release.yml`

## Still present (OK / deferred)

| Item | Why left |
|------|----------|
| `apps/frontend`, blog, docs, issue templates for web/API | Upstream Modrinth website stack; not the Owyx product binary |
| `steve_head.png` on launcher-files CDN | Generic Steve skin thumbnail, not Modrinth logo |
| Locales other than EN/RU still saying “Modrinth App” in places | Cheap follow-up; EN/RU + defaults fixed for product chrome |
| Hosting / friends / shared-instance copy mentioning Modrinth servers | Describes Modrinth’s cloud features still in code paths; ads/hosting chrome largely disabled |
| Instance icon `wrench.png` | Generic tool icon catalog entry (label renamed from “Modrinth Wrench”) — not the labyrinth mark |
| Disabled upstream workflows (`frontend-deploy`, `theseus-release` CDN) | Gated to `modrinth` org / never run on this fork |

## Release

Use workflow **Owyx GitHub Release** (tag `v*` or workflow_dispatch). Artifacts: Windows NSIS + Linux AppImage on GitHub Releases only.
