# Minecraft / Loader meta sources

Официальные списки версий (не наш backend). Используем их для экрана «Настройка установки».

## Minecraft client JARs

1. Манифест всех версий:  
   `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`
2. У нужной версии взять поле `url` → JSON этой версии.
3. Client jar: `downloads.client.url` (+ `sha1`, `size`).  
   **Owyx / Theseus layout:** store under shared  
   `%USERPROFILE%/owyx/meta/versions/{id}/{id}.jar` (+ `{id}.json`),  
   **not** under `libraries/com/mojang/minecraft/client.jar`.  
   Libraries: `libraries[]` → обычно `https://libraries.minecraft.net/...` → `meta/libraries/`  
   Assets: `assetIndex` → `https://resources.download.minecraft.net/...` → `meta/assets/`

Документация: [wiki.vg Game files](https://wiki.vg/Game_files), [Minecraft Wiki version_manifest](https://minecraft.wiki/w/Version_manifest.json).

Старый хост `launchermeta.mojang.com` ещё отвечает, канонический — **piston-meta**.

## Loaders

| Loader | Meta |
|--------|------|
| Fabric | `https://meta.fabricmc.net/v2/versions/game`, `.../loader/{mc}` |
| Quilt | `https://meta.quiltmc.org/v3/versions/loader/{mc}` |
| Forge | Maven metadata `https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml` (версии вида `1.20.1-47.2.0`) |
| NeoForge | `https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml` |
| Vanilla | только Mojang version manifest |

## CurseForge packs (future)

Импорт zip/папки: `manifest.json` (projectID/fileID модов) + `overrides/` (`config`, `kubejs`, …) + иногда HTML index со ссылками.  
Скачивание модов — через CurseForge API / mirror (нужен API key на backend). Не в MVP главного меню.

## Owyx control-plane backend (future)

Docker на mini-PC → FRPC → публичный IP:port VPS.  
Админский лаунчер пишет настройки, игровые лаунчеры читают.  
Дружеский лист сборок — **после** классического install/play flow.
