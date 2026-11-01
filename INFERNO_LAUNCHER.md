# Inferno Launcher

A PvP-focused Minecraft launcher, forked from the open-source
[Modrinth App](https://github.com/modrinth/code) (Tauri + Vue + Rust). It keeps the Modrinth App
engine (accounts, Java management, instances, mod downloads, dependency resolution) and adds:

- **Arena**: the new home screen with one-click, pre-built PvP profiles.
- **Inferno Client bundled**: the 1.21.10 and 1.21.11 jars ship inside the installer
  (`apps/app/resources/inferno/`) and are added to each profile automatically.
- **A different look**: warm charcoal + ember theme, flame branding/icon, square tab navigation with
  labels, new typography treatment (`apps/app-frontend/src/assets/stylesheets/inferno-theme.scss`).
- **No ads**: Modrinth's ad webview and consent flow are disabled (`src/helpers/ads.js`).

## Profiles

Each profile is a Fabric instance for Minecraft 1.21.11 or 1.21.10 (the versions Inferno ships for).
Mods are resolved live from Modrinth when you press **Build profile**, so you always get the newest
compatible build, with Fabric API pulled in as a dependency. If a mod has no build for that
Minecraft version it is skipped and reported - it never fails the whole profile.

| Profile | For | Mods (besides Inferno Client) |
| --- | --- | --- |
| Inferno Duels | All-round PvP | Sodium, Lithium, ModernFix, Entity Culling, ImmediatelyFast, Sodium Extra, Mod Menu |
| Max FPS Arena | Crowded fights, low-end PCs | Sodium, Lithium, ModernFix, FerriteCore, Entity Culling, ImmediatelyFast, More Culling, Dynamic FPS, Sodium Extra, Mod Menu |
| Clean Slate | Strict server/tournament lists | Sodium |
| Clip Lab | Content creators | Sodium, Lithium, ModernFix, Iris, Entity Culling, ImmediatelyFast, Mod Menu |

No hacked-client or cheat mods are included, and a test enforces that no cheat-style slug is added.
Edit `apps/app-frontend/src/features/pvp/profiles.ts` to change or add profiles.

**Server rules:** Inferno Client includes Freelook and Fullbright. Many servers allow them; some
do not. Check the rules of where you play and turn them off in the Inferno menu if needed.

## Building

Requirements: Node 24+, pnpm, Rust (stable), and on Windows the MSVC build tools + WebView2.
On Linux: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev libxdo-dev`.

```bash
pnpm install
cp packages/app-lib/.env.prod packages/app-lib/.env
pnpm i18n:coverage
pnpm --filter @modrinth/app dev      # run in development
pnpm --filter @modrinth/app build    # produce installers in target/release/bundle
```

The GitHub Actions workflow `.github/workflows/inferno-build.yml` builds Windows, macOS and Linux
installers (run it from the Actions tab, or push a `v*` tag).

Tests for the profile installer and profile data (no Tauri needed):

```bash
cd apps/app-frontend
npx --yes esbuild@0.27.2 tests/pvp.test.ts --bundle --platform=node --format=cjs --outfile=pvp.test.cjs
node --test pvp.test.cjs
```

## What is and is not verified

Verified in development: TypeScript type-check, production frontend build, the installer logic
(mocked backend, 8 tests), and visual rendering of the new UI in dark and light themes.

**Not yet verified:** a full native build and an end-to-end run (create profile -> download mods ->
launch Minecraft). Those need a desktop machine or the CI workflow. The one piece most worth
watching on first run is resolving the bundled jar path (`inferno/...` via Tauri resources).

## Before you publish it

- **License:** this is a fork of GPL-3.0 software, so your distribution must stay GPL-3.0 and ship
  its source. Keep `COPYING.md` / `LICENSE` and the attribution above.
- **Not affiliated with Modrinth.** Do not use Modrinth's name or logo for your launcher. Some
  internal identifiers and the deep-link scheme (`modrinth://`) are unchanged, so it can clash with a
  separately installed Modrinth App. Your data lives in its own folder (`InfernoLauncher`).
- **Microsoft sign-in** uses the same public client ID as the upstream app (and most third-party
  launchers). For a public release, register your own Azure app and set `MICROSOFT_CLIENT_ID` in
  `packages/app-lib/src/state/minecraft_auth.rs`.
- **Inferno Client** is bundled as supplied; its `fabric.mod.json` lists a placeholder contact URL.
  Make sure you have the right to redistribute it.
- Auto-update is not configured (the upstream updater points at Modrinth's servers and is not enabled).
