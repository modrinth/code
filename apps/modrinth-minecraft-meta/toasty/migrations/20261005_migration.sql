ALTER TABLE "forge_installers" ADD COLUMN "processed_at" TIMESTAMPTZ(6);
CREATE TABLE "quilt_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_quilt_catalogs_by_download_run_id" ON "quilt_catalogs" ("download_run_id");
CREATE TABLE "neo_forge_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "forge_sha256" TEXT NOT NULL,
    "neoforge_sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_neo_forge_catalogs_by_download_run_id" ON "neo_forge_catalogs" ("download_run_id");
