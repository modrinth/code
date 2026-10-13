CREATE TABLE "minecraft_versions" (
    "name" TEXT NOT NULL,
    "catalog_entry" JSON NOT NULL,
    "manifest" JSON NOT NULL,
    "source_catalog_sha256" TEXT NOT NULL,
    "source_manifest_sha256" TEXT NOT NULL,
    "is_latest_release" BOOLEAN,
    "is_latest_snapshot" BOOLEAN,
    PRIMARY KEY ("name")
);
CREATE UNIQUE INDEX "index_minecraft_versions_by_is_latest_release" ON "minecraft_versions" ("is_latest_release");
CREATE UNIQUE INDEX "index_minecraft_versions_by_is_latest_snapshot" ON "minecraft_versions" ("is_latest_snapshot");
