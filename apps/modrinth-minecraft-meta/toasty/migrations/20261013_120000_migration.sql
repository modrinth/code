ALTER TYPE "fabriclike_loader" ADD VALUE 'ornithe';
CREATE TABLE "ornithe_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "games_sha256" TEXT NOT NULL,
    "loaders_sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_ornithe_catalogs_by_download_run_id" ON "ornithe_catalogs" ("download_run_id");
