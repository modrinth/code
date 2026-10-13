Game version and launcher version related metadata manifests.

# Architecture

The general flow of the metadata service is:
- Import
  - Download metadata and manifests from upstream sources (Mojang, Fabric, NeoForge, etc.), and save them into storage as raw JSON blobs.
  - Take the JSON blobs and extract structured information out of them, useful for our purposes, and save that in our database.
  - Do other extraction, like reading JSON manifests embedded in installer JAR files, and save that in our database.
  - Download required Maven artifacts, or extract them from installer JARs, into content-addressed storage. Record their hashes alongside the structured metadata.
  - If an upstream's downloads fail, keep its previous structured metadata and continue with other upstreams. Commit new structured metadata only after its required artifacts are stored. Report failures at the end of the run.
- Export
  - Read all entries in our database, and turn them into manifests used by the app.
  - Upload these manifests, and stored Maven artifacts, into the local filesystem or S3. Export never downloads from upstream sources.
  - Publish referenced artifacts and version manifests before publishing the overall catalog.
  - Historical catalogs can contribute versions absent from newer catalogs. Raw import history is retained, while selected structured metadata can be replaced when upstream information changes.

This means that:
- Every time we import, we save the source manifests that we download - from this, we can derive any information that we want without needing to query the upstreams again.
- We have a historical record of all upstream manifests - if an upstream goes down, we're not affected and can keep using old information until it's back up.
- Imports and exports can be rerun. This does not guarantee that overlapping runs are safe; run them sequentially.
- Much of our data is effectively immutable, simplifying things a lot - no need to deal with missing versions if we never delete versions.
- Given the immutability, we can track the provenance of each entity (game version, loader version, etc.)
- We still have the option to hide information by not including it in the final manifest upload - e.g. if we no longer want to show a version, we can mark it as `unlisted` in our database, and the final manifest creation won't include it, but it'll still exist in the DB.

## Running

From the repository root:

```sh
cargo run -p modrinth-minecraft-meta -- import
cargo run -p modrinth-minecraft-meta -- export
```

Without upstream flags, all upstreams are selected. Put flags before the command to select specific upstreams:

```sh
cargo run -p modrinth-minecraft-meta -- --fabric --quilt import
cargo run -p modrinth-minecraft-meta -- --fabric --quilt export
```

Forge and NeoForge currently import installer metadata and embedded artifacts, and export the embedded artifacts. Their launcher manifests are not implemented yet.

## Forge and NeoForge

Forge and NeoForge (henceforth Forgelike) store much of their important info in the installer JAR file, rather than as a JSON file served at some HTTP route.

## Ornithe

Ornithe runs Fabric Loader on game versions up to 1.14.4, with its own intermediary (`net.ornithemc:calamus-intermediary-gen2`). Its profiles look like Fabric's, so it is extracted and exported through the same code, but a single template doesn't work:
- Ornithe adds different libraries depending on the game version (e.g. a newer log4j, or LWJGL builds from Legacy Fabric's Maven), which aren't in its profiles. Game versions are grouped by these libraries, and profiles are downloaded once per group, templated from the group's longest game version. The libraries are added to every profile of their group.
- The LWJGL builds come from Ornithe's copy of each version manifest. They keep their upstream downloads, like Mojang's libraries.
- Ornithe's version manifests also drop some of Mojang's libraries (e.g. `org.ow2.asm:asm-all`). These are listed in the profile's `removedLibraries`, and split groups without changing which profile is downloaded.
- Group IDs are hashed the same way as the legacy daedalus client, so they stay stable when switching over.
- A few game versions are named differently by Ornithe and Mojang (e.g. `1.0.0` and `1.0`). These get a group of their own whose profile isn't templated, since the app would fill the placeholder with Mojang's name.
- Only game versions in Mojang's catalog are exported, so Mojang metadata must be extracted first.

## Fabric and Quilt

Fabric's and Quilt's manifests contain (among other things) two fields:
- `game`, a list of Minecraft game versions, like `26.4-snapshot-2`
- `loader`, a list of Fabric loader versions, like `0.19.5`

For each pair of game and loader version, Fabric/Quilt serve a file at the path `/v2/versions/loader/{minecraft_version}/{loader_version}/profile/json`. However, it would be impractical to download and mirror every single file here, especially considering that for any given `loader_version`, the `profile/json` file doesn't change between `minecraft_version`s - usually (foreshadowing). Therefore, instead of downloading every `(minecraft_version, loader_version)` pair, we instead download every `(1.21, loader_version)` pair - that is, `minecraft_version` is always 1.21[^1], but `loader_version` varies. We fix up this downloadad manifest and replace `1.21` with `${modrinth.gameVersion}`; then when a client downloads our manifests, they replace `${modrinth.gameVersion}` back to whatever game version they want to play.

However, this doesn't always work. Specifically with Quilt, and Minecraft version 26 and later, there was a change to how the `hashed` field is treated, so we need special behavior here. This is where version groups come in.

TODO explain version groups

[^1]: Don't ask me why we picked 1.21 specifically, I don't know.
