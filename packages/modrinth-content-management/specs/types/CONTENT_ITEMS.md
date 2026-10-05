# Content item

A content item represents a mod, resource pack, shader pack or data pack within a content set.

Each set has its own items, even when they use the same files. File details are stored in the [content entry](./CONTENT_ENTRY.md).

- `id`
- `content_set_id`: ID of the [content set](./CONTENT_SET.md) this item belongs to.
- `enabled`: Whether the item is enabled.
- `locked`: Whether individual version changes are prevented.
- `project_type`: Mod, resource pack, shader pack, or data pack.
- `project`: Optional, Modrinth project ID.
- `version`: Optional, Modrinth version ID belonging to `project`. Requires `project` to be set.
- `date_added`: When the item was **added to this set.**
- `source_kind`: Source of the content. See [Content sources](#content-sources).
- `environment`: V3 environment type of the content item. See [Environments](#environments).
- `owner`: Optional, project owner details. See [Project owner details](#project-owner-details).
- `has_update`: Whether an update is known to be available.
- `update_version_id`: Optional, modrinth version ID to update to.
- `embedded_metadata`: Optional, name, version, and icon read from the file. See [Embedded metadata](#embedded-metadata).
<!-- - `synced_pack`: Optional pack sync details: ID, participating instance IDs, and whether an update is pending. -->

## Environments

The environment uses the Modrinth v3 values:

- ClientAndServer
- ClientOnly
- ClientOnlyServerOptional
- SingleplayerOnly
- ServerOnly
- ServerOnlyClientOptional
- DedicatedServerOnly
- ClientOrServer
- ClientOrServerPrefersBoth
- Unknown

`Unknown`: Environment is unavailable.

## Project owner details

The project owner details must be:

- `id`
- `name`
- `avatar_url`
- `type`: User or Organization

## Content sources

See [CONTENT_SOURCES.md](./CONTENT_SOURCES.md)

## Embedded metadata

When both `project` and `version` are unset, the file is external or has not been identified.

Optional metadata from the file:

- `name`
- `version`: Version label
- `icon_path`: Path to an icon found in the file.
