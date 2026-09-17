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
3. SkillsMP: admin/dashboard/list inspiration only — implement in **existing** Vue/Next components, do not paste HTML dashboard templates.
4. Launcher UI: reuse `@modrinth/ui` **`Toggle` / `ToggleCard`** (and settings layout patterns used by Appearance/Privacy), not bare checkboxes that disappear on dark themes.
5. i18n: `defineMessages` + FormatJS for launcher; site locales `en_US` / `ru_RU` (+ `merge-site-i18n.mjs` if needed).

---

## Priority A — Launcher Social settings

**File:** `apps/app-frontend/src/components/ui/settings/account/SocialSettings.vue`

### Canonical field name

Use **`sharePresence` only** (not `presenceEnabled`) everywhere: SQL `share_presence`, PATCH/GET JSON, TS helpers, UI, contract.

### Must fix / keep

1. **Allow friend requests** — `@modrinth/ui` `Toggle`; wire `getOwyxSocialSettings` / `patchOwyxSocialSettings` (`allowFriendRequests`); optimistic + revert on error; toggle sound.
2. **Presence OFF (`sharePresence`)**
   - **ON:** heartbeat ~30s; idle ~90s → offline to friends.
   - **OFF:** stop heartbeats; force offline to friends; badge Off.
   - Backend: `user_social_settings.share_presence` (migration `013_share_presence.sql` + `ensureFriendsSchema`); `GET|PATCH /api/friends/settings`.
   - When disabled: `POST /api/friends/presence` **forces offline** (never leave stale online); friends list masks via `share_presence`.
   - Turning OFF in PATCH immediately writes offline presence.
   - Client: `owyx-presence` + Social toggle stay in sync; **do not wipe `playing` → `online`** when re-entering Social or refreshing session while already sharing.
   - Update `LAUNCHER_SITE_CONTRACT.md`.
3. **Visual polish** — density like Appearance/Privacy; hierarchy header → stats → toggles → skins; RU/EN; no Modrinth product wording beyond CSL Modrinth project link.
4. **Anti-regress locks (#29) — non-negotiable**
   - `openFriends` **must** call `settingsModal?.close()` via `inject(appSettingsModalContextKey)` before `router.push('/')` and `owyx:open-friends`.
   - Presence badge / `presenceLive` = `owyxPresenceStatus` ∈ `{online, playing}` — **not** `isSignedIn`.
   - Pending chip = only `status === 'pending' && incoming` (label Incoming).
   - Signed-out empty state still prompts Owyx sign-in.
   - Do not break CSL install flow.

---

## Priority B — Logs & telemetry

Already on `main` from #35: migration `012_logs_telemetry.sql`, telemetry ingest, AdminLogs, `owyx-telemetry.ts`, support CTA.

### Finish / harden

1. Log important account mutations via `activityLog.js` (`logUserActivity`); never passwords/tokens/raw emails. Prefer `req.clientIp` (trust-proxy path) — do not spoof via raw `X-Forwarded-For`.
2. Admin Account tab: searchable/filterable/human labels.
3. Launcher telemetry opt-in `settings.telemetry`; no PII; Admin Launcher KPI + filters + empty states.
4. Document VPS apply for `012` (+ `013`); fail soft if table missing.
5. Cover skin delete and other account mutations still missing.

---

## Priority C — Site UI

Profile Security/Profile full-width; liquid glass + `prefers-reduced-transparency`; `SupportContact.tsx` → GitHub new issue.

## Priority D — Release / CI hygiene

Do not regress `minecraft_auth.rs` soft-fail (`unwrap_err`, not `expect_err`). Fix intl/theme AR/CI issues.

## Out of scope

Hosting/Medal/Modrinth+; MC plugin; mass `@modrinth/*` rename; secrets/`.env`/binaries.

---

## Working loop (AR + Grok babysitter)

1. Implement Priority A first (Social), then B, then C/D.
2. Push to **this PR branch** only (`cloud/polish-logs-social-ideal` → `ebluffy/Owyx`).
3. After each meaningful push: wait for CI + Autoreview; fix everything actionable.
4. Grok babysitter nags using **this document** as SSoT — address every open item until closed.
5. Re-read this brief before declaring done; update the checklist below.

---

## Acceptance checklist

- [x] Social: visible Toggle for allow friend requests; persists correctly
- [x] Social: `sharePresence` GET/PATCH; OFF stops heartbeat; force-offline ingest; friends see offline; badge Off
- [x] Social: layout matches settings design system; no invisible controls; RU/EN OK
- [x] Anti-regress #29: modal `close()`, presence ≠ `isSignedIn`, pending = incoming only
- [x] Presence engine does not wipe `playing` when Social loads / session refreshes while sharing
- [x] Skins / CSL install still works
- [x] Admin Logs: account + moderation + launcher tabs polished (human labels)
- [x] Telemetry: opt-in only (fresh default off; existing TRUE kept); rate-limited ingest; no PII; contract + `012`/`013` VPS notes
- [x] `logUserActivity` uses trusted `req.clientIp`; skin_delete logged
- [x] Site glass/layout/support verified (SupportContact → GitHub issue; profile full-width + liquid glass on main/#35)
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
| Migrations | `012_logs_telemetry.sql`, `013_share_presence.sql` |
| Contract | `owyxsite/LAUNCHER_SITE_CONTRACT.md` |
| Brand | `brand/DESIGN.md`, `AGENTS.md` |
