# Content set

A content set represents a playable Minecraft instance or server.

It contains everything you need to install minecraft, a mod loader and any content items. They are designed to be easily comparable to other content sets.

- `id`: Content set ID.
- `name`: Display name.
- `source_kind`: Source of the set.
- `status`: `Available`, `Installing`, `Stale`, or `MissingFiles`.
- `game_version`: Minecraft version.
- `protocol_version`: Optional Minecraft protocol number.
- `loader`: Mod loader, or vanilla.
- `loader_version`: Optional loader version.
- `created`: Creation time.
- `modified`: Last modification time.
