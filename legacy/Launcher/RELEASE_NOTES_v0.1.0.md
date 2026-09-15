# Owyx Launcher v0.1.0 — Release Notes (draft)

**Status:** draft for first public/private drop. Do **not** publish the GitHub Release until smoke on Windows is signed off.

## Highlights

- Classic launcher flow: profile → create instance → Play installs → launch
- Shared Minecraft cache under `%USERPROFILE%/owyx/meta/` (Theseus-style):
  - `meta/versions/{id}/` — version JSON + client jar
  - `meta/libraries/` — libraries
  - `meta/assets/` — assets
  - `meta/natives/{version}/` — natives
- Fabric / Quilt / Forge / NeoForge / Vanilla
- Home · Library · instance detail (Content / Files / Worlds / Logs)
- Instance settings modals (memory presets, Java, hooks) with autosave
- RU / EN i18n
- Modrinth mods search **scaffold** (Content → Browse mods) — API wired, marketplace polish next

## Critical fixes in this RC

- Client jar no longer lands under a fake `libraries/com/mojang/minecraft/client.jar` path
- Install no longer freezes ~2 minutes at `N-1/N` after `client.jar` reaches 100% (SHA1 O(n²) bug)
- Existing wrong-path client jars are adopted into `meta/versions/` when size matches
- Play no longer re-hashes every jar with SHA256 before launch
- Gear and ⋮ actions next to Play are equal square buttons; ⋮ includes **Open instance folder**

## Artifact

```text
Build (Windows):
  cd Launcher/apps/launcher
  npm run tauri build

Ship path:
  src-tauri/target/release/Owyx.exe

Smoke copy (documented):
  Launcher/Owyx.exe
```

Data root: `%USERPROFILE%/owyx/`

## Known gaps (post-v0.1)

- Real Microsoft / Owyx browser auth (offline nick works for Play)
- Full Modrinth marketplace (install from search, versions, dependencies)
- Friends pack sync / control-plane
- CI Windows release build
- Autoupdate EXE checks (updater shell exists)

## Smoke checklist

1. Create offline profile
2. Create vanilla instance → Play → install completes without hang on last file
3. Second instance same MC version reuses `meta/versions` (no re-download of client)
4. Stop / exit → Play button returns (not stuck “running”)
5. ⋮ → Open instance folder
6. Content → Browse mods → search returns Modrinth hits
7. Switch tabs during install → top progress pill stays consistent
