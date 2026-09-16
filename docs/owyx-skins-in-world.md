# Owyx skins in-world (CustomSkinLoader)

## Goal

Players signed into Owyx (or using the same offline nickname as their site account) should see **their** skin and **other Owyx players’** skins in Minecraft, without relying on Mojang/Microsoft skin servers.

## Chosen path

Reuse **[CustomSkinLoader](https://github.com/xfl03/MCCustomSkinLoader)** (Modrinth: [customskinloader](https://modrinth.com/mod/customskinloader)).

| Item | Value |
|------|--------|
| License | **GPL-3.0** |
| Loaders | Fabric, Forge, NeoForge, Quilt (per upstream) |
| Type | Client-side mod (do not install on dedicated servers) |
| Trademark | Do **not** ship TLauncher / TLSkinCapes proprietary stacks |

## Site API (public)

No `X-Owyx-Client-Key` required when called from the website host; `/api/csl/*` is also exempted on `api.*` so the in-game mod can fetch textures.

| Endpoint | Purpose |
|----------|---------|
| `GET /api/csl/skins/{nickname}.png` | Legacy PNG for CSL |
| `GET /api/csl/{nickname}.json` | CustomSkinAPI profile JSON |
| Skin upload | Existing profile cosmetics on owyx.site (`LAUNCHER_SITE_CONTRACT`) |

Prefer configuring CSL with **`https://owyx.site/api/csl/`** so Minecraft clients never need the launcher client key.

## Launcher wiring

1. On Owyx site login, cosmetics are synced to `~/owyx/skins/{nickname}.png` (`owyx-cosmetics.ts`).
2. On instance launch (and via **Instance → Installation → Write Owyx skin config**), the launcher writes `CustomSkinLoader/CustomSkinLoader.json` pointing at Owyx Legacy + CustomSkinAPI entries (`owyx-csl.ts`).
3. LocalSkin mirror copies `~/owyx/skins/{nick}.png` into the instance for offline fallback.
4. **Player still must install the CustomSkinLoader mod jar** into the instance mods folder (Modrinth link in Social settings / instance options).

## Compatibility notes

- Works best when in-game username matches the Owyx site nickname (offline / Owyx account path).
- Microsoft accounts keep Mojang skins via CSL’s MojangAPI fallback after Owyx sources.
- Capes: site field exists; CSL cape entry is reserved / optional.

## Not in this sprint

- Auto-downloading the CSL jar into every instance (Modrinth dependency install UX).
- Server-side SkinRestorer plugin (useful for pure vanilla clients — later).
