# Content set

A content set represents a playable Minecraft setup for an instance or server.

It contains the Minecraft version, loader and content items.

- `id`
- `instance_id`: Optional, local instance this set belongs to. Required for sets with local content entries.
- `name`: Display name.
- `source_kind`: Source of the set. See [Content set sources](#Content-set-sources)
- `status`: See [Status](#status).
- `game_version`: Minecraft version.
- `protocol_version`: Optional, Minecraft protocol number. See [Protocol number](#protocol-number).
- `loader`: Mod loader, or `vanilla`.
- `loader_version`: Optional, loader version; required unless the loader is `vanilla`.
- `created`: Creation time.
- `modified`: Last modification time.

## Status

- `NotInstalled`: Not installed
- `Available`: Ready to use
- `Installing`: Installation in progress.
- `Stale`: The installed setup needs updating. (Used by shared instances)
- `MissingFiles`: A needed file has no entry, is missing or has a conflict - ready to use but might break on launch.

## Content items

Items belong to the set through `content_set_id`. Entries belong to their item through `content_item_id`. File paths must be unique within a set.

An instance can have multiple sets, with one applied at a time.

## Content set sources

See [CONTENT_SOURCES.md](./CONTENT_SOURCES.md).

The source kind identifies the type of source, not a particular project or server.

## Protocol number

The vanilla protocol number used by this content set’s Minecraft version. It can be used to quickly identify the game protocol independently of its version name.

See [Minecraft Wiki: Protocol Version, Java Edition](https://minecraft.wiki/w/Protocol_version#Java_Edition)
