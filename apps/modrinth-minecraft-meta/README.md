Game version and launcher version related metadata manifests.

# Architecture

The general flow of the metadata service is:
- Download metadata and manifests from upstream sources (Mojang, Fabric, NeoForge, etc.), and save them into our database, as raw JSON blobs.
- Take the JSON blobs and turn them into more structured information, useful for our purposes, and save that in our database.
- Upload our processed manifests into some publicly-accessible source, such as the local filesystem, or an S3-compatible service like Cloudflare R2.

This means that:
- Every time we run the service, we save the source manifests that we download - from this, we can derive any information that we want without needing to query the upstreams again.
- We have a historical record of all upstream manifests - if an upstream goes down, we're not affected and can keep using old information until it's back up.
- Much of our data is effectively immutable, simplifying things a lot - no need to deal with missing versions if we never delete versions.
- Given the immutability, we can track the provenance of each entity (game version, loader version, etc.)
- We still have the option to hide information by not including it in the final manifest upload - e.g. if we no longer want to show a version, we can mark it as `unlisted` in our database, and the final manifest creation won't include it, but it'll still exist in the DB.

## Forge and NeoForge

Forge and NeoForge (henceforth Forgelike) store much of their important info in the installer JAR file, rather than as a JSON file served at some HTTP route.

## Fabric and Quilt

Fabric's and Quilt's manifests contain (among other things) two fields:
- `game`, a list of Minecraft game versions, like `26.4-snapshot-2`
- `loader`, a list of Fabric loader versions, like `0.19.5`

For each pair of game and loader version, Fabric/Quilt serve a file at the path `/v2/versions/loader/{minecraft_version}/{loader_version}/profile/json`. However, it would be impractical to download and mirror every single file here, especially considering that for any given `loader_version`, the `profile/json` file doesn't change between `minecraft_version`s - usually (foreshadowing). Therefore, instead of downloading every `(minecraft_version, loader_version)` pair, we instead download every `(1.21, loader_version)` pair - that is, `minecraft_version` is always 1.21[^1], but `loader_version` varies. We fix up this downloadad manifest and replace `1.21` with `${modrinth.gameVersion}`; then when a client downloads our manifests, they replace `${modrinth.gameVersion}` back to whatever game version they want to play.

However, this doesn't always work. Specifically with Quilt, and Minecraft version 26 and later, there was a change to how the `hashed` field is treated, so we need special behavior here. This is where version groups come in.

TODO explain version groups

[^1]: Don't ask me why we picked 1.21 specifically, I don't know.
