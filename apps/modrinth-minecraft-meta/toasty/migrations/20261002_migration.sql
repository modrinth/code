DROP TABLE "minecraft_versions";
DROP TABLE "download_blobs";
DROP TABLE "json_blobs";
CREATE TABLE "mojang_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE TABLE "blob_hashes" (
    "sha256" TEXT NOT NULL,
    "sha1" TEXT NOT NULL,
    PRIMARY KEY ("sha256")
);
CREATE INDEX "index_blob_hashes_by_sha1" ON "blob_hashes" ("sha1");
CREATE TABLE "blob_downloads" (
    "download_run_id" UUID NOT NULL,
    "url" TEXT NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("download_run_id", "url")
);
CREATE INDEX "index_blob_downloads_by_sha256" ON "blob_downloads" ("sha256");
