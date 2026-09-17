# Cloud agent brief — polish to ideal (logs, telemetry, Social settings)

**Repo:** `ebluffy/Owyx` (fork of `modrinth/code`; keep fork relationship).  
**Base:** `main` (includes merged #35 — admin Logs + launcher telemetry foundation).  
**This PR:** work here until Autoreview (AR) + Grok babysitter are green and product is ideal.  
**Do not** open PRs against `modrinth/code` upstream. Target **`ebluffy/Owyx` only**.

---

## Mission (non-negotiable)

Polish and finish — do **not** leave foundations half-done. Iterate until:

1. **Social settings** in the launcher look and behave like a finished product (toggles work, presence can be turned off, layout matches Owyx brand / design system).
2. **Logs + telemetry** are complete end-to-end: useful site account events, admin UX, launcher stats/errors (no PII), contract docs, migration applied notes.
3. **Support** CTA remains correct (site modal → GitHub new issue); extend only if natural.
4. **Release path** stays green (0.8.1 failed on `MinecraftToken` / `expect_err` — already fixed on main; do not regress).
5. Run the **AR ↔ fix ↔ Grok babysitter** loop until both are satisfied. Prefer skills from [ebluffy/agent-tool-catalog](https://github.com/ebluffy/agent-tool-catalog) + SkillsMP for UI/admin patterns. Respect `brand/DESIGN.md`, `AGENTS.md`, `owyxsite/LAUNCHER_SITE_CONTRACT.md`, SemVer (`0.2.0` baseline; next launcher cut likely `0.8.2` after green CI).

Stop only when the PR is merge-ready: green CI, AR clean (or only intentional deferred notes), babysitter satisfied, no obvious UX holes on Social / Logs.

---

## Skills / tooling (required)

1. Read `.agents/skills/agent-tool-catalog/SKILL.md` and the live catalog README.
2. Prefer catalog UI defaults: **hallmark** + **ui-ux-pro-max** — still **Owyx tokens first** (`brand/DESIGN.md`, cyan `#00e5ff`, bg `#050508`, Sora/Onest/Unbounded). No Modrinth green-as-primary, no Hosting upsell chrome.
3. SkillsMP: admin/dashboard/list inspiration only (e.g. FlowAI activity-log patterns) — implement in **existing** Vue/Next components, do not paste HTML dashboard templates.
4. Launcher UI: reuse `@modrinth/ui` **`Toggle` / `ToggleCard`** (and settings layout patterns used by Appearance/Privacy), not bare checkboxes that disappear on dark themes.
5. i18n: `defineMessages` + FormatJS for launcher; site locales `en_US` / `ru_RU` (+ `merge-site-i18n.mjs` if needed).

---

## Priority A — Launcher Social settings (broken / ugly today)

**File:** `apps/app-frontend/src/components/ui/settings/account/SocialSettings.vue`  
**Screenshot context (v0.8.0):** Settings → Social looks unfinished: stats row OK-ish; Presence is a read-only badge with **no way to disable**; “Allow friend requests” has **no visible switch** (native checkbox is effectively invisible); skins section works.

### Must fix

1. **Allow friend requests**
   - Replace native `<input type="checkbox">` with design-system **`Toggle`** (or `ToggleCard`) so the control is obvious on dark themes.
   - Keep wire to `getOwyxSocialSettings` / `patchOwyxSocialSettings` (`allowFriendRequests`).
   - Saving must remain optimistic + revert on error; play toggle sound if that pattern exists.

2. **Presence — user must be able to turn it OFF**
   - Today presence is display-only (`owyxPresenceStatus` → Active/Off badge). That is not enough.
   - Add a real setting, e.g. `sharePresence` / `presenceEnabled` (name consistently across API + client):
     - **ON:** current heartbeat behavior (~30s while signed in; idle ~90s → offline to friends).
     - **OFF:** stop heartbeats; force offline to friends; clear/stop presence updates; badge reflects Off.
   - Backend: extend `user_social_settings` (migration if needed) + `GET|PATCH /api/friends/settings` in `owyxsite/backend/src/routes/friends.js`. When disabled, `POST /api/friends/presence` should no-op or set offline; friends list must not show the user as online.
   - Client: `owyx-presence` helper + Social toggle must stay in sync.
   - Update `LAUNCHER_SITE_CONTRACT.md`.

3. **Visual polish (Social tab)**
   - Remove “max-w-lg left-stuck empty space” feel; use the same settings density as Appearance/Privacy.
   - One clear hierarchy: header → stats → toggles (Presence, Allow requests) → skins.
   - No cluttered card soup; use ToggleCard / settings rows consistently.
   - Stats chips: keep useful, but refine spacing/typography to match brand.
   - Skins block: keep “Install CustomSkinLoader” + “Open profile on site” working; tighten copy/layout only.
   - RU/EN strings complete; no leftover Modrinth wording.

4. **Regression**
   - Friends panel open-from-settings still works.
   - Signed-out empty state still prompts Owyx sign-in.
   - Do not break CSL install flow.

---

## Priority B — Logs & telemetry (finish beyond foundation)

Already on `main` from #35:

- Migration `owyxsite/postgres/migrations/012_logs_telemetry.sql`
- `POST /api/launcher/v1/telemetry`, admin `GET /api/admin/activity|logs|telemetry`
- Admin UI `AdminLogs.tsx`, launcher `owyx-telemetry.ts`, support CTA, glass/layout fixes

### Finish / harden

1. **Site account activity completeness**
   - Ensure every important account mutation is logged in `user_activity` (login/logout, register, login change, email change, password change, display nick, avatar, profile fields, ban-related if applicable). Use `owyxsite/backend/src/utils/activityLog.js`; never log passwords/tokens/raw emails in metadata (redact).
   - Admin Account tab: searchable, filterable, readable labels (humanize types if needed).

2. **Launcher telemetry usefulness**
   - Keep opt-in via `settings.telemetry`.
   - Events: session_start, heartbeat, error/crash, optional perf/feature — expand only if useful.
   - PC stats: OS, version, arch, cpu cores, RAM estimate — **no** usernames, emails, absolute home paths, tokens.
   - Wire more high-signal error paths if missing (startup failures, auth soft-fails summary without secrets).
   - Admin Launcher tab: KPI row + filters solid; empty states clear.

3. **Ops**
   - Document VPS apply step for `012_logs_telemetry.sql` in site README / contract if not already obvious.
   - Fail soft if table missing (503 with clear message is OK).

4. **Privacy**
   - Double-check sanitizers client + server.
   - No PII in admin telemetry table by design.

---

## Priority C — Site UI polish (carry-over)

- Profile Security/Profile cards: full-width layout already started — verify no left-stuck empty gutters; inputs use full content column.
- Liquid glass: visible but not gaudy; respect `prefers-reduced-transparency`.
- Support button: modal copy + GitHub issue link (`components/support/SupportContact.tsx`) — keep working; optionally add to admin/cabinet if it fits without clutter.

---

## Priority D — Release / CI hygiene

- Do not regress `packages/app-lib/src/state/minecraft_auth.rs` soft-fail path (use `unwrap_err`, not `expect_err` on non-Debug types).
- Fix any intl/key-order / theme enum issues AR or CI report.
- If cutting a launcher release after merge: SemVer patch/feature per `.cursor/rules/semver.mdc`; sync versions via `scripts/set-app-version.js`; prefer **0.8.2** (or next appropriate) after green — only if asked or clearly needed to validate.

---

## Out of scope

- Modrinth Hosting / Medal / Modrinth+ upsell.
- Minecraft plugin / game-token revival.
- Mass-renaming `@modrinth/*` packages.
- Committing secrets, `.env`, real client keys, binaries.

---

## Working loop (AR + Grok babysitter)

1. Implement Priority A first (Social), then B, then C/D.
2. Push to **this PR branch** only (`cloud/polish-logs-social-ideal` → `ebluffy/Owyx`).
3. After each meaningful push: wait for CI + Autoreview; fix everything actionable.
4. Grok babysitter will nag on this PR using **this document** as the source of truth — address every open item until closed.
5. Re-read this brief before declaring done; update the PR checklist below.

---

## Acceptance checklist

- [x] Social: visible Toggle for allow friend requests; persists correctly
- [x] Social: presence can be disabled; friends see offline; heartbeat stops
- [x] Social: layout matches settings design system; no invisible controls; RU/EN OK
- [x] Skins / CSL install still works
- [x] Admin Logs: account + moderation + launcher tabs polished and useful
- [x] Telemetry: opt-in only; no PII; contract updated
- [x] Site glass/layout/support verified
- [ ] CI green on `ebluffy/Owyx`
- [ ] AR clean (or only approved deferrals documented in PR)
- [ ] Grok babysitter satisfied / no open blockers

---

## Key paths

| Area | Path |
|------|------|
| Social UI | `apps/app-frontend/src/components/ui/settings/account/SocialSettings.vue` |
| Presence client | `apps/app-frontend/src/helpers/owyx-presence.ts` |
| Friends API client | `apps/app-frontend/src/helpers/owyx-friends.ts` |
| Friends/presence API | `owyxsite/backend/src/routes/friends.js` |
| Telemetry client | `apps/app-frontend/src/helpers/owyx-telemetry.ts` |
| Telemetry ingest | `owyxsite/backend/src/routes/launcher.js` |
| Admin logs UI | `owyxsite/frontend/components/admin/AdminLogs.tsx` |
| Activity helper | `owyxsite/backend/src/utils/activityLog.js` |
| Migration | `owyxsite/postgres/migrations/012_logs_telemetry.sql` |
| Contract | `owyxsite/LAUNCHER_SITE_CONTRACT.md` |
| Brand | `brand/DESIGN.md`, `AGENTS.md` |
