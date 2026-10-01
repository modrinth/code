DROP INDEX "index_download_blobs_by_download_run_id_and_url";
ALTER TABLE "download_blobs" ADD COLUMN "id" UUID NOT NULL;
CREATE UNIQUE INDEX "index_download_blobs_by_download_run_id_and_url" ON "download_blobs" ("download_run_id", "url");
CREATE UNIQUE INDEX "index_download_blobs_by_id" ON "download_blobs" ("id");
CREATE TABLE "minecraft_versions" (
    "id" UUID NOT NULL,
    "source_blob_id" UUID NOT NULL,
    PRIMARY KEY ("id")
);
