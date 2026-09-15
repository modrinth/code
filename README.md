# Owyx

Public fork of [modrinth/code](https://github.com/modrinth/code) used as the **Owyx desktop launcher** (and related Modrinth-stack packages).

**Personal / friends use for the next ~2 years** — not a commercial Modrinth competitor. We follow upstream updates and keep **Owyx branding** (no Modrinth trademarks).

## Layout

| Path | What |
|------|------|
| `apps/app` | Tauri shell → **Owyx** binary |
| `apps/app-frontend` | Launcher UI (Vue) |
| `packages/app-lib` | Theseus-style launcher core (Rust) |
| `apps/frontend`, `apps/labrinth`, … | Upstream packages (web/API) — keep for sync; Owyx site lives in `legacy/owyxsite` for now |
| `legacy/` | Previous private monorepo pieces: old Tauri launcher, Paper plugin, control-plane site |

Old private monorepo history: **https://github.com/ebluffy/OwyxOld**

## Branding (COPYING.md)

Removed / must not ship Modrinth logos, cover art, wrench-in-labyrinth marks, or green brand-as-primary.  
Product name in the app shell: **Owyx**. Accent target: cyan `#00e5ff` (see `legacy/DESIGN.md`).

`api.modrinth.com` as a **content API host** stays — that is not their product trademark UI.

## Dev (launcher)

See upstream Theseus / app docs. Typical:

```bash
pnpm install
pnpm app:dev
```

## Upstream

```bash
git remote add upstream https://github.com/modrinth/code.git   # if missing
git fetch upstream
git merge upstream/main   # or rebase — resolve brand conflicts carefully
```

## License

Upstream package licenses apply. Respect Modrinth branding restrictions in `COPYING.md`.
