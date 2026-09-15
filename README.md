# Owyx

**Owyx** desktop Minecraft launcher — personal / friends use (~2 years). Built on a public fork of [modrinth/code](https://github.com/modrinth/code) so we can pull upstream fixes; **product branding is Owyx**, not Modrinth.

Brand kit: [`brand/`](./brand/) (v2 logos + [`brand/DESIGN.md`](./brand/DESIGN.md)). Accent `#00e5ff`, bg `#050508`.

Old private monorepo (site / old launcher / plugin archive): **https://github.com/ebluffy/OwyxOld**

## Layout

| Path | What |
|------|------|
| `apps/app` | Tauri shell → **Owyx** binary |
| `apps/app-frontend` | Launcher UI (Vue) |
| `packages/app-lib` | Launcher core (Rust) |
| `brand/` | Owyx logos, hero art, design tokens |
| `owyxsite/` | Owyx control-plane (site + API + deploy) — our code, not upstream |
| `apps/frontend`, `apps/labrinth`, … | Upstream packages kept for sync |

## Branding

No Modrinth logos, cover art, or green-as-primary in the shipping app.  
`api.modrinth.com` may still be used as a **content API host** — that is not product trademark UI.

## Dev (launcher)

```bash
pnpm install
pnpm app:dev
```

## Upstream

```bash
git remote add upstream https://github.com/modrinth/code.git   # if missing
git fetch upstream
git merge upstream/main   # resolve brand conflicts carefully
```

## License

See repository `LICENSE` / `COPYING.md` (AGPL and trademark constraints from upstream apply to the fork).
