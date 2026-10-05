# Content set

A content set represents a playable Minecraft instance or server.

It contains everything you need to install minecraft, a mod loader and any content items. They are designed to be easily comparable to other content sets.

- `id`: Content set ID.
- `name`: Display name.
- `source_kind`: Source of the set. See [Content set sources](#Content-set-sources)
- `status`: `Available`, `Installing`, `Stale`, or `MissingFiles`.
- `game_version`: Minecraft version.
- `protocol_version`: Optional, Minecraft protocol number. See [Protocol-number](#protocol-set-number)
- `loader`: Mod loader, or `vanilla`.
- `loader_version`: Optional, loader version - required if not `vanilla` loder.
- `created`: Creation time.
- `modified`: Last modification time.

## Content set sources

See [CONTENT_SOURCES.md](./CONTENT_SOURCES.md)

## Protocol number

The vanilla protocol number used by this content set’s Minecraft version. It can be used to quickly identify the game protocol independently of its version name.

See [Minecraft Wiki: Protocol Version, Java Edition](https://minecraft.wiki/w/Protocol_version#Java_Edition)
