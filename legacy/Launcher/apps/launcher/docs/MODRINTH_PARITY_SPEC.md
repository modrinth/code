# Full Modrinth App parity map → Owyx surfaces

SPEC for `cursor/modrinth-full-parity-*`. Source of truth: Modrinth `apps/app-frontend` + `packages/ui` (clone at `/tmp/modrinth-code`, not committed). Brand: Owyx `DESIGN.md` tokens. Control-plane: `api.owyx.site` + client key preserved.

## Skills used (no SkillSpector)

| Skill | Why |
|-------|-----|
| github/spec-kit (spec-driven) | This screen map before implement |
| mattpocock/skills | Engineering / requirements discipline |
| addyosmani/agent-skills | Spec → implement → ship |
| nutlope/hallmark | Anti-slop pass on large screens |
| nextlevelbuilder/ui-ux-pro-max | UX anti-patterns on Home/Browse/Content |

## Screen map

| Modrinth surface | Modrinth path | Owyx route / surface | Status |
|------------------|---------------|----------------------|--------|
| App shell (rail, titlebar, crumbs, accounts, downloads) | `App.vue` | `#shell` rail + topbar + history + status pill | Port |
| Home / Index | `pages/Index.vue` | `route=home` `renderHome` | Port density + Owyx news |
| Discover / Browse (global) | `pages/Browse.vue` `/browse/*` | `route=browse` (no instance) | Port |
| Browse for instance | Browse + `?i=` | `route=browse` + `browseInstanceId` | Port (Content CTA) |
| Project detail | `pages/project/*` | `route=project` + projectId | Port |
| Library | `components/ui/library/*` | `route=library` | Port filters/toolbar |
| Instance layout + header | `pages/instance/layout.vue` | `route=instance` | Port |
| Content (installed) | `instance/content` + ContentPageLayout | Instance tab `content` | Port — no inline search |
| Files / Worlds / Logs | `instance/files|worlds|logs` | Instance tabs | Keep + polish |
| Share | `instance/share` | Tab `share` | Honest stub (no Modrinth SaaS) |
| Create instance | CreationFlowModal | Multi-step `#modal-create` | Port stages |
| Skins | `pages/Skins.vue` | `route=skins` | Port spirit + Owyx `/me` apply |
| Screenshots (global) | `pages/Screenshots.vue` | Optional / deferred | Not blocker |
| Hosting / Modrinth+ | hosting/* | **Omit** | Replaced by Owyx Servers |
| Servers (Modrinth) | `pages/Servers.vue` | **Owyx Servers** `route=servers` | Keep API catalog |
| App settings | AppSettingsModal | `#modal-app-settings` | Expand sections |
| Instance settings | settings-modal/** | `#modal-inst-settings` | Keep tabs |
| Accounts | AccountsCard + MS auth | Profile wizard + Playing as | Offline + Owyx + MS B |
| Play / install / logs | app-lib | Rust `install`/`launch`/`modrinth` | Keep |

## Left rail (target)

1. Logo Owyx (hex)  
2. Home  
3. Discover / Browse  
4. Skins  
5. **Servers (Owyx)** — private catalog (not Modrinth Hosting)  
6. Separator + instance icons (recent)  
7. `+` create  
8. Settings  
9. Playing-as / avatar  

Stats moves out of primary Modrinth-like rail (reachable from Home/Settings if kept).

## Content → Browse contract

- Content tab: search installed · Upload files · **Browse content** (primary) · type chips · list with enable/delete.  
- **No** eternal inline Modrinth search panel as primary UX.  
- Browse content → `route=browse` with locked MC/loader from instance.  
- Card click → `route=project` (Description / Gallery / Versions).  
- Install always uses explicit `projectId` / `versionId`.

## Create wizard stages (Modrinth-aligned, Owyx order)

Owner baseline: **loader → MC → loader version → name → icon** (real multi-step panels, not a 2-panel collapse).

1. Loader chips  
2. Minecraft version  
3. Loader version (vanilla skips with honest hint)  
4. Name  
5. Icon (optional) → Create  

Alt: **Browse modpacks** / **Import .mrpack** — Discover install + local file import create an instance (game jars still on Play). Export from Share is overrides-based (partial).

## Project detail tabs

Description · Gallery · **Changelog** (from version changelogs) · Versions.

## Auth

| Kind | Target |
|------|--------|
| Offline | Keep |
| Owyx email | `api.owyx.site` + `X-Owyx-Client-Key` |
| Microsoft | Path **A** when `MICROSOFT_CLIENT_ID` set (device-code → Xbox → MC); else `unconfigured`. See `docs/MICROSOFT_AUTH.md`. |

## Explicit non-goals / honest gaps

- Modrinth Hosting, Modrinth+, ads, wrench logo, green brand accent  
- Friends / shared-instance cloud SaaS  
- Full Theseus content DB / mrpack CDN-hash export / Mojang skin equip + capes  
- Screenshots global page — optional  
- MS token use in Play launch args / DPAPI wrapping of `ms1:` blobs  

## Modrinth source files used (primary)

- `apps/app-frontend/src/App.vue`  
- `pages/Index.vue`, `Browse.vue`, `Skins.vue`  
- `pages/instance/layout.vue`, `content/index.vue`, `files|worlds|logs`  
- `pages/project/Index.vue` (+ Description/Gallery/Versions)  
- `packages/ui/.../content-tab/layout.vue`, `browse-tab/*`  
- `packages/ui/.../creation-flow-modal/**`  
- `packages/app-lib` (auth/install patterns referenced for MS B docs)

## Verification

```bash
cd Launcher/apps/launcher && npm run check && npm run check:rust && npm run build:smoke
```
