# Agent notes (Owyx monorepo)

Canonical plan: [`PLAN.md`](./PLAN.md). Design: [`DESIGN.md`](./DESIGN.md).
Launcher ↔ site API: [`owyxsite/LAUNCHER_SITE_CONTRACT.md`](./owyxsite/LAUNCHER_SITE_CONTRACT.md).
Launcher build gates: [`Launcher/BUILD.md`](./Launcher/BUILD.md).

Do not expand local `Launcher/PLAN.md` / `owyxsite/PLAN.md` — they only point here.
Do not commit `.env`, secrets, `*.exe`, `target/`, `node_modules/`, DB dumps.

## Cursor Cloud specific instructions

This section is for agents that boot after the update script (npm installs only).
Standard commands live in the READMEs above — do not duplicate install steps here.

**Product split.** `owyxsite/` is the control-plane (Next `:3000` + Express `:3001` +
Postgres). `Launcher/` is the client: **native Tauri** on Linux (WebKitGTK) and
Windows (WebView2), plus Vite `:1420` for fast UI-only preview.
`owyxplugin/` is out of friends-sprint scope.

**Site DB.** Compose wants Postgres **17**. Cloud VMs often have **16** via apt —
that is enough for local API work. Existing Docker volumes do **not** auto-apply
new SQL (`initdb.d` runs once). Apply `owyxsite/postgres/migrations/007_servers_packs.sql`
with `psql -f` on old volumes. Backend also runs `ensureCatalogSchema()` on boot;
those tables must be **owned by** the API role (`owyx_user`). If created as
`postgres`, startup fails with `42501 must be owner of table packs`.
Launcher-only PRs should **not** edit `owyxsite/` unless the task is the
launcher↔site / VPS contour (this file’s site notes still apply).

**Launcher — Cloud Linux (primary).** After `bash Launcher/scripts/linux-dev-setup.sh`
(WebKitGTK 4.1 / GTK / Rust ≥ 1.85), from `Launcher/apps/launcher`:

- **Primary:** `npm run dev:tauri` — Vite + **native WebKit window**; Rust
  invoke (catalog, install, skin-to-disk) is live. Needs `DISPLAY` (`:1` or `:0`).
- **Secondary / fast UI:** `OWYX_API_BASE_URL=http://127.0.0.1:3001 npm run dev`
  — browser `:1420`; Download/Play/skin-to-disk are stubs (no invoke).
- **Compile gate:** `npm run check && npm run check:rust`
- **Smoke:** `npm run build:smoke` (or `build:smoke:linux`) → ELF
  `src-tauri/target/debug/owyx`. Window opens; logs under `~/owyx/`.
- **Windows Play / ship exe:** hand-off after merge (MiniMax on Windows).
  Do **not** treat Wine + `Owyx.exe` as the cloud path — WebView2 stays blank.

**Frontend lint.** `owyxsite/frontend` `npm run lint` should be green after the
quality-pass AuthProvider / profile / admin effect fixes. If it fails, it is a
regression — do not ignore set-state-in-effect on auth.

**Site + launcher together (Cloud).** Prefer Docker when the daemon is up:

```bash
cd owyxsite && bash deploy/vps-up.sh --dev   # or: cp .env.example .env && docker compose up -d --build
export OWYX_API_BASE_URL=http://127.0.0.1:3001
cd ../Launcher/apps/launcher && npm run dev:tauri
```

`deploy/vps-up.sh --dev` sets `NODE_ENV=development` so `TURNSTILE_SKIP=true`
works. Without `--dev` (VPS): loopback binds + host nginx for `owyx.site` /
`api.owyx.site`; `TURNSTILE_SECRET_KEY` is required (`SKIP` is ignored in
production). **Do not** add `panel.owyx.site` (C³ CELERITY). See
`owyxsite/deploy/README.md` and `owyxsite/deploy/OBT_PREP.md`. Apt
Postgres 16 without Docker is still OK for API-only work.

**Hot reload.** Backend `ensureCatalogSchema` only seeds when `packs`/`servers`
are empty. After changing seed SQL, insert/update rows yourself or use admin UI.
Restart Express after route changes; Vite/Next pick up UI edits.

## AutoReview (Cursor Automation)

Cloud PRs should get **AutoReview** comments on the latest commit. If it does
not run on push:

1. Prefer **one commit + one push** at the end of work (see
   `.cursor/rules/cloud-agent-prompts.mdc`).
2. After push, wait **~10 minutes** before marking the PR merge-ready.
3. Re-authorize the **Cursor GitHub App** on `ebluffy/Owyx` if webhooks broke
   after the repo rename.
4. In Cursor Automations, trigger should include **PR synchronize**, not only
   “PR opened”.
5. Manual **TEST** with the PR URL is an acceptable fallback; cite the run in
   the PR comment.

Do not stack many commits in seconds — AutoReview may attach only to the last
SHA or skip until manual TEST (seen on PR #13: 3 commits in ~4s, review only
after manual TEST on `0d70ac0`).
