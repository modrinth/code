CREATE TYPE "catalog_kind" AS ENUM ('mojang', 'fabric', 'forge', 'neoforge', 'quilt');
DROP TABLE "mojang_catalogs";
CREATE TABLE "catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "kind" "catalog_kind" NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_catalogs_by_download_run_id_and_kind" ON "catalogs" ("download_run_id", "kind");
