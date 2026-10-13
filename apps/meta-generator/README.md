Game version and launcher version related metadata manifests.

The generator reuses models, upstream schemas, and shared utilities from the `modrinth-minecraft-meta` package. Import/export orchestration, configuration, storage, and migrations remain in `meta-generator`.

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
cargo run -p meta-generator -- import
cargo run -p meta-generator -- export
```

Without upstream flags, all upstreams are selected. Put flags before the command to select specific upstreams:

```sh
cargo run -p meta-generator -- --fabric --quilt import
cargo run -p meta-generator -- --fabric --quilt export
```

Forge and NeoForge currently import installer metadata and embedded artifacts, and export the embedded artifacts. Their launcher manifests are not implemented yet.

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
