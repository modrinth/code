DROP TABLE "catalogs";
CREATE TABLE "forge_installers" (
    "name" TEXT NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("name")
);
CREATE TABLE "forge_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_forge_catalogs_by_download_run_id" ON "forge_catalogs" ("download_run_id");
CREATE TABLE "mojang_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_mojang_catalogs_by_download_run_id" ON "mojang_catalogs" ("download_run_id");
CREATE TABLE "fabric_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_fabric_catalogs_by_download_run_id" ON "fabric_catalogs" ("download_run_id");
