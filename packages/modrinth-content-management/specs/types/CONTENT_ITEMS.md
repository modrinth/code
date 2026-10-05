## Content item

A content item represents a mod, resource pack, shader pack or data pack within a content set.

- `id`: File hash
- `file_name`: Filename.
- `file_path`: File path
- `size`: File size in bytes.
- `enabled`: Whether the item is enabled.
- `locked`: Whether the item is locked against individual changes. See [Locking](#Locking)
- `project_type`: Mod, resource pack, shader pack, or data pack.
- `project`: Optional, Modrinth project id
- `version`: Optional, Modrinth version id
- `environment`: Environment type of the content item, see [Environments](#Environments)
- `owner`: Project owner details, see [Project owner details](#Project-owner-details)
- `has_update`: Whether an update is available.
- `update_version_id`: Optional, modrinth version ID to update to.
- `date_added`: Optional, date shown for the item.
- `source_kind`: Optional, source of the content. See [Content sources](#Content-sources)
- `embedded_metadata`: Optional, name, version, and icon read from the file. See [Embedded metadata](#Embedded-metadata)
- `synced_pack`: Optional pack sync details: ID, participating instance IDs, and whether an update is pending.

## Locking

Locking a content item will:
- Prevent it's `version` from being changed through either:
	- Updating flow
	- Switch version flow

## Environments

The environment of a content item can be one of the following:

- ClientAndServer
- ClientOnly
- ClientOnlyServerOptional
- SingleplayerOnly
- ServerOnly
- ServerOnlyClientOptional
- DedicatedServerOnly
- ClientOrServer
- ClientOrServerPrefersBoth
- Unknown (never used)

## Project owner details

The project owner details must be:

- `id`
- `name`
- `avatar_url`
- `type`: User or Organization

## Content sources

See [CONTENT_SOURCES.md](./CONTENT_SOURCES.md)

## Embedded metadata

When both `project` and `version` are not defined, this means it's an external file.

External files will have metadata about them inside of the file which must be extracted:

- `name`
- `version`: Version label
- `icon_path`: Path to an icon found in the file.


### Locations

- **Fabric mods:** Read `fabric.mod.json` from the JAR root. Extracts `name` (falling back to `id`), `version`, and `icon`. [Fabric docs](https://docs.fabricmc.net/develop/loader/fabric-mod-json)
- **Quilt mods:** Read [`quilt.mod.json`](https://github.com/QuiltMC/rfcs/blob/main/specification/0002-quilt.mod.json.md). Extract the name from `quilt_loader.metadata.name` (falling back to `quilt_loader.id`), the version from `quilt_loader.version`, and the icon from `quilt_loader.metadata.icon` (falling back to `quilt_loader.icon`).
- **NeoForge mods:** Read `META-INF/neoforge.mods.toml`. Extract the mod’s `displayName` (falling back to `modId`), `version`, and `logoFile`. [NeoForge docs](https://docs.neoforged.net/docs/gettingstarted/modfiles/)
- **Forge mods:** Read `META-INF/mods.toml` for those fields. Older Forge mods can use `mcmod.info`. [Forge docs](https://docs.minecraftforge.net/en/1.12.x/gettingstarted/structuring/)
- **Spigot and Paper plugins:** Their JARs can contain `plugin.yml` or `paper-plugin.yml` with plugin metadata, including name and version. [Spigot docs](https://www.spigotmc.org/wiki/plugin-yml/), [Paper docs](https://docs.papermc.io/paper/dev/plugin-yml/)

If a mod’s name or version is missing from its loader metadata, you may use `Implementation-Title` or `Implementation-Version` from `META-INF/MANIFEST.MF`.

For resource and data packs look for `pack.png` but do not extract name or version from `pack.mcmeta`
