# legacy/ — previous Owyx monorepo (archived in-tree)

Frozen snapshot of the **private** stack before switching to the Modrinth App fork.

| Path | Was |
|------|-----|
| `Launcher/` | Custom Tauri 2 + Vite launcher (`main.ts`) |
| `owyxsite/` | Next + Express + Postgres control-plane |
| `owyxplugin/` | Paper/Purpur plugin |
| `DESIGN.md` / `PLAN.md` | Product design & plan |

Authoritative old git remote: https://github.com/ebluffy/OwyxOld

Do **not** treat this as the shipping launcher anymore. Use root `apps/app` + `apps/app-frontend`.

`.env` files here (if present) are local-only and gitignored.
