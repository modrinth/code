# Cloud sprint brief — close UX / practice debts (site + launcher)

**Repo:** https://github.com/ebluffy/Owyx  
**Updated:** 2026-09-17  
**Audience:** Cursor Cloud Agent (branch + PR only; do not merge `main`)

This is the **tracked** brief. Paste task lives locally in gitignored `promt.md`.

---

## Already shipped on `main` (do not redo)

- Owyx™ branding, `COPYING.md` / `TRADEMARK.md`, no “private Minecraft” in site tab title (`Owyx™` only).
- Site public EN/RU: home, download shell, auth pages, profile, servers marketing, coming-soon, legal docs, header/footer locale toggle.
- Launcher OBT line through **0.5.2** (offline active policy, etc.). Window title in source is already `Owyx™` (`apps/app/tauri.conf.json`).
- Friends / catalog control-plane + `owyxsite/LAUNCHER_SITE_CONTRACT.md` (API ~1.2.0).

---

## Goal of this sprint

Close **usage + practice debts** so EN/RU and everyday flows feel finished for friends — not new product pillars (no plugin rewrite, no Microsoft OAuth completion, no Modrinth marketplace redesign).

**Definition of Done**

1. Public site: switching **EN** leaves **no** user-visible Russian outside intentional locale strings (admin may stay RU-primary if timeboxed; prefer EN/RU parity if feasible).
2. Launcher: account chooser (offline / Owyx / MS stub) and Servers tab empty/error states are clear in **en-US + ru-RU**; support links do not dump users onto Modrinth support by default.
3. No secrets committed; no touch of `apps/frontend` / `apps/labrinth` for product work.
4. Branch pushed + `gh pr create` (or `PR_BODY.md` fallback). Brief section **“Agent result”** updated at end of PR.

---

## Priority backlog (do in order)

### P0 — Site i18n holes

| Item | Where | Notes |
|------|--------|--------|
| Hardcoded RU on download page | `owyxsite/frontend/app/download/page.tsx` | Skin/account CTA still RU when EN |
| News cards locale gaps | `components/sections/NewsSection.tsx` | Badge class keys / RU fallbacks |
| Admin panel RU-only | `app/admin/page.tsx`, `components/admin/CatalogAdmin.tsx` | Wire through `locales/*` like profile; staff UI OK if bilingual |
| LegalDoc / meta polish | `components/legal/LegalDoc.tsx`, `app/legal/[slug]/page.tsx` | Prefer locale keys over ternary; titles `… — Owyx™` |
| Grep pass | `owyxsite/frontend` | Cyrillic outside `locales/ru_RU.json` and admin if deferred |

### P1 — Launcher everyday UX

| Item | Where | Notes |
|------|--------|--------|
| Error / Minecraft-required support URLs | `ErrorModal.vue`, `MinecraftRequiredModal.vue` | Prefer owyx.site / Discord / honest “no Modrinth support”; keep package names `@modrinth/*` |
| Welcome + account paths | `WelcomeScreen.vue`, auth UI | Three paths obvious: offline nick, Owyx site account, MS (stub honesty) |
| Servers empty / unreachable | Owyx Servers pages + `owyx.*` locale keys | en-US + ru-RU messages; no dead “Modrinth Hosting” chrome |
| Settings copy | Privacy / about strings | No “we are Modrinth”; upstream attribution OK in legal tone |

### P2 — Practice / hygiene (small, high value)

- Confirm `SkipLink` + focus order on auth/profile; form labels/`aria-*` on login/register.
- Site: `days with us` / date formatting consistent with locale (profile already uses `formatDate`).
- Launcher: do **not** cut a GitHub Release unless a real fix warrants PATCH; SemVer via `.cursor/rules/semver.mdc`.
- After site UI changes: note owner must `ssh owyxsite` → `/opt/owyx` pull + `owyxsite/deploy/vps-up.sh` (cloud agent usually cannot deploy prod).

### Out of scope

- Rebranding `@modrinth/*` package names.
- Full Microsoft OAuth.
- Plugin / old `OwyxOld` stack.
- Redesigning Modrinth content discovery / ads systems.
- Force-push, secrets, merging own PR to `main`.

---

## Sources of truth

1. Root `AGENTS.md`, `brand/DESIGN.md`, `TRADEMARK.md`, `COPYING.md`
2. `owyxsite/LAUNCHER_SITE_CONTRACT.md`, `owyxsite/frontend/AGENTS.md`
3. `packages/ui/AGENTS.md` only if touching shared UI (prefer not)
4. External tooling catalog: `.agents/skills/agent-tool-catalog/` → https://github.com/ebluffy/agent-tool-catalog

### Suggested external skills (SkillsMP / catalog — read, don’t vendor wholesale)

| Need | Skill |
|------|--------|
| Site a11y | [frontend-a11y](https://skillsmp.com/creators/affaan-m/ecc/skills-frontend-a11y) / [accessibility](https://skillsmp.com/creators/affaan-m/ecc/skills-accessibility) |
| Next/React patterns | [frontend-patterns](https://skillsmp.com/creators/affaan-m/ecc/agents-skills-frontend-patterns), [react-performance](https://skillsmp.com/creators/affaan-m/ecc/skills-react-performance) |
| Vue launcher UI | [vue-patterns](https://skillsmp.com/creators/affaan-m/ecc/skills-vue-patterns); repo skill `.agents/skills/i18n-pass/` for Vue string migration |
| Tauri specifics | [tauri-v2](https://skillsmp.com/skills/midudev-autoskills-packages-autoskills-skills-registry-tauri-v2-skill-md) |
| Process | [writing-plans](https://skillsmp.com/skills/obra-superpowers-skills-writing-plans-skill-md), [requesting-code-review](https://skillsmp.com/skills/obra-superpowers-skills-requesting-code-review-skill-md) |
| UI anti-slop | hallmark + ui-ux-pro-max from agent-tool-catalog — still obey `brand/DESIGN.md` |

Repo skills win over generic SkillsMP when they conflict.

---

## Agent result (fill at end of PR)

- Branch:
- PR URL:
- Done:
- Deferred:
- Risks / owner follow-up (deploy, release):
