# apps/launcher — Owyx Launcher (Tauri 2)

Skeleton for the desktop client. Product plan: repository root [`PLAN.md`](../../../PLAN.md).

## Dev

```powershell
# from repo root
copy .env.example .env   # once

cd apps\launcher
npm install

# UI in browser (HTML/CSS) — screenshots without native build
npm run dev
# open http://localhost:1420

# Full app (WebView2 + Rust), hot reload
npm run tauri dev

# Fast verify (preferred): typecheck → debug smoke build
npm run check
npm run check:rust
npm run build:smoke
# -> src-tauri/target/debug/owyx.exe
# optional: Copy-Item to ../../Owyx.debug.exe

# Ship only (slow): release exe for hand-off
npm run build:release
# -> src-tauri/target/release/Owyx.exe → copy ../../Owyx.exe
# see ../../BUILD.md
```

Requires: Rust stable, Node 20+, WebView2 (Edge runtime, same family as Modrinth App).

Shared Minecraft cache: `%USERPROFILE%/owyx/meta/versions|libraries|assets` (Theseus layout).

## What this PR includes

- Vite + TypeScript UI shell (nick field, packs placeholder, Play disabled until nick)
- Rust `greet` command to verify Tauri bridge
- Tauri window config for Windows

Next (see PLAN stages 1–3): config.json, update API client, sync + SHA256, Fabric launch.
