# Cloud sprint brief — playable-release polish (site + launcher)

**Repo:** https://github.com/ebluffy/Owyx  
**Updated:** 2026-09-17  
**Audience:** Cursor Cloud Agent (branch + PR only; do not merge `main`)

Tracked brief. Paste prompt: local gitignored `promt.md`.

---

## Already on `main` (do not redo from scratch)

- Owyx™ branding, `COPYING.md` / `TRADEMARK.md`, site tab title **Owyx™** only.
- Site public EN/RU shell (home/auth/profile/servers/legal/header locale).
- Launcher **0.5.2** OBT; window title `Owyx™` in `apps/app/tauri.conf.json`.
- Friends API + presence contract in `owyxsite/LAUNCHER_SITE_CONTRACT.md` (~1.2.0).
- Skin upload API on site (cosmetics migration); launcher Social UI still largely placeholder.
- Public GitHub fork + shipped OBT/Beta builds — product is already released as Beta, not a private invite app.

---

## Goal

Reach a **playable public Beta bar**: finish Social/Friends/Account/Share, wire skins into the world (TLSkin-class or thin custom mod), close i18n/UX debts, optional practical product ideas, then **UI/UX polish to shine**. Repo is **public** — anyone can download/use; do **not** market or write copy as “owner’s friends only / private circle.” Friends = social feature, not exclusivity. Owner is out of product ideas — agent invents a useful next backlog in the PR.

**Not in scope:** merge to main, GitHub Release, secrets, `apps/frontend`/`labrinth` product work, full MS OAuth, OwyxOld plugin, Modrinth marketplace redesign, **prod site deploy** (owner will ask a local agent later).

---

## Definition of Done

1. Settings → Social is not a wall of “coming soon”; friends + presence usable with site API.
2. Account paths clear (offline / Owyx / MS stub); Share invites don’t dead-end.
3. Skin visibility path for Owyx players documented + MVP (reuse TLSkin-compatible open mod **or** thin custom mod + launcher install).
4. Public site EN has no stray RU; launcher support links don’t default to Modrinth support.
5. Final UI/UX polish pass (motion, optional open-licensed UI sounds with mute, modern brand-faithful look).
6. PR includes agent-written **Ideas / next** backlog; Agent result section filled below.
7. No deploy attempts; PR notes owner deploys after merge.

---

## Priority backlog

### A — Finish Social / Friends / Account / Share

| Area | Reality today | Target |
|------|---------------|--------|
| Friends panel | API helpers exist | Add/accept/decline/remove + empty/error states en+ru |
| Presence | Heartbeat in contract (~90s) | Show online/playing/offline in UI |
| Social settings | Placeholders in Settings → Social | Real toggles where API allows; else honest disabled + reason |
| Account | Three auth paths | Grandma-clear copy; no Modrinth support URLs |
| Share | Shared instance invites | Accept/decline/install playable |

### B — In-game skins (TLauncher-like)

- Prefer **open TLSkinCapes-compatible** (or similar) mod if license OK; wire skin URL from Owyx profile/API.
- Or thin **custom mod scaffold** + auto-install into instances from launcher.
- Document license, MC/loader versions, install path in PR.
- Do not ship proprietary TLauncher or misuse trademarks.

### C — Site / launcher practice debts

- Hardcoded RU: `download/page.tsx`, `NewsSection.tsx`, admin if timeboxed.
- Launcher: `ErrorModal` / `MinecraftRequiredModal` support URLs; Servers empty states; de-Modrinth user copy.

### D — Invent practical product (optional 1–3)

Only if useful to **admin** (catalog, ACL, publish, diagnostics) or **all players**. Small MVPs only; list each in PR.

### E — UI/UX polish pass (last)

- Modernize within `brand/DESIGN.md`.
- Light animations + `prefers-reduced-motion`.
- Optional UI SFX from **open** sources; cite license/URL; settings mute.
- Empty states, loading, focus, density — site + launcher.

### F — PR Ideas section (required)

Agent fills: can add / need polish / launcher still needs / site still needs / risks.

---

## Sources of truth

`AGENTS.md`, `brand/DESIGN.md`, `TRADEMARK.md`, `COPYING.md`, `owyxsite/LAUNCHER_SITE_CONTRACT.md`, `.agents/skills/agent-tool-catalog/`, repo `i18n-pass`.

SkillsMP picks: frontend-a11y, frontend-patterns, vue-patterns, tauri-v2, writing-plans, requesting-code-review; UI via catalog hallmark / ui-ux-pro-max after brand tokens.

---

## Agent result (fill at end of PR)

- Branch: `cursor/playable-release-polish-e12e`
- PR URL: https://github.com/ebluffy/Owyx/pull/14
- Done (A–E): A Social/Friends/Account/Share playable; B CSL skin path + docs; C public EN scrub + launcher support links; D copy-playing / API health / UI sounds; E presence pulse, skeletons, reduced-motion
- Skin approach: CustomSkinLoader (GPL-3.0) + public `/api/csl` + launcher config writer — see `docs/owyx-skins-in-world.md`
- Deferred: auto-install CSL jar; full admin i18n; MS OAuth; SkinRestorer server plugin
- Ideas summary: in PR body “Ideas / next (agent)”; also `docs/playable-release-polish-plan.md`
- Owner follow-up: **deploy site after merge** (local agent); optional launcher release bump suggestion: **0.5.3 PATCH** or **0.6.0 MINOR** (owner chooses; cloud did not cut a GitHub Release)

