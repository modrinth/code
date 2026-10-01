Game version and launcher version related metadata manifests.

# Architecture

The general flow of the metadata service is:
- Download metadata and manifests from upstream sources (Mojang, Fabric, NeoForge, etc.), and save them into our database, as raw JSON blobs.
- Take the JSON blobs and parse them into more structured information, according to the upstream's schema, then save that to our database.
- Take the structured upstream info and turn them into more structured game versions, loader versions, etc., which we also store in our database.
- Upload our processed manifests into some publicly-accessible source, such as the local filesystem, or an S3-compatible service like Cloudflare R2.

This means that:
- Every time we run the service, we save the source manifests that we download - from this, we can derive any information that we want without needing to query the upstreams again.
- We have a historical record of all upstream manifests - if an upstream goes down, we're not affected and can keep using old information until it's back up.
- Much of our data is effectively immutable, simplifying things a lot - no need to deal with missing versions if we never delete versions.
- Given the immutability, provenance of each entity (game version, loader version, etc.) in our DB can be finely tracked.
- We still have the option to hide information by not including it in the final manifest upload - e.g. if we no longer want to show a version, we can mark it as `unlisted` in our database, and the final manifest creation won't include it, but it'll still exist in the DB.

## Upstream sources

For Minecraft, our basic building blocks are:
- `MinecraftVersion`, a single version of the Minecraft game as released by Mojang, such as 26.3 or 25w46a.
- `MinecraftLoader`, an enum of the kinds of Minecraft mod loaders we support (Fabric, Forge, etc.)
- `MinecraftLoaderVersion`, a single version of a mod loader release, such as NeoForge 47.1.106 or Fabric 0.19.5.

All of the above data is gathered from upstreams and stored locally. We don't serve a mirror of this data like some other launchers, but we use it for processing into manifests that our app then uses to launch the game. We also mark whether one of these entities is no longer present in an upstream, but if it disappears from our upstream we don't delete it from our local database. This gives us an archive of all upstream manifests, and lets us keep serving an old version even if it's deleted from the upstream (in case the upstream misbehaves or has maintenance issues).

##
