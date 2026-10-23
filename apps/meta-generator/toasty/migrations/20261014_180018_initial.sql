CREATE TYPE "fabriclike_loader" AS ENUM ('fabric', 'quilt');
CREATE TYPE "minecraft_loader" AS ENUM ('fabric', 'forge', 'neoforge', 'quilt');
CREATE TYPE "forgelike_loader" AS ENUM ('forge', 'neoforge');
CREATE TABLE "blob_hashes" (
    "sha256" TEXT NOT NULL,
    "sha1" TEXT NOT NULL,
    PRIMARY KEY ("sha256")
);
CREATE INDEX "index_blob_hashes_by_sha1" ON "blob_hashes" ("sha1");
CREATE TABLE "fabric_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_fabric_catalogs_by_download_run_id" ON "fabric_catalogs" ("download_run_id");
CREATE TABLE "fabriclike_profiles" (
    "loader" "fabriclike_loader" NOT NULL,
    "version_group" TEXT NOT NULL,
    "loader_version" TEXT NOT NULL,
    "stable" BOOLEAN NOT NULL,
    "position" INTEGER NOT NULL,
    "source_url" TEXT NOT NULL,
    "source_sha256" TEXT NOT NULL,
    "template" TEXT NOT NULL,
    "metadata" JSON NOT NULL,
    PRIMARY KEY ("loader", "version_group", "loader_version")
);
CREATE TABLE "quilt_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_quilt_catalogs_by_download_run_id" ON "quilt_catalogs" ("download_run_id");
CREATE TABLE "forge_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_forge_catalogs_by_download_run_id" ON "forge_catalogs" ("download_run_id");
CREATE TABLE "blob_downloads" (
    "download_run_id" UUID NOT NULL,
    "url" TEXT NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("download_run_id", "url")
);
CREATE INDEX "index_blob_downloads_by_sha256" ON "blob_downloads" ("sha256");
CREATE TABLE "neoforge_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "forge_sha256" TEXT NOT NULL,
    "neoforge_sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_neoforge_catalogs_by_download_run_id" ON "neoforge_catalogs" ("download_run_id");
CREATE TABLE "forgelike_installers" (
    "loader" "forgelike_loader" NOT NULL,
    "name" TEXT NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("loader", "name")
);
CREATE INDEX "index_forgelike_installers_by_sha256" ON "forgelike_installers" ("sha256");
CREATE TABLE "mojang_catalogs" (
    "id" UUID NOT NULL,
    "download_run_id" UUID NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_mojang_catalogs_by_download_run_id" ON "mojang_catalogs" ("download_run_id");
CREATE TABLE "forgelike_extracts" (
    "installer_sha256" TEXT NOT NULL,
    "created_at" TIMESTAMPTZ(6) NOT NULL,
    "metadata" JSON NOT NULL,
    "embedded_maven_artifacts" JSON NOT NULL,
    PRIMARY KEY ("installer_sha256")
);
CREATE TABLE "download_runs" (
    "id" UUID NOT NULL,
    "started_at" TIMESTAMPTZ(6) NOT NULL,
    "completed_at" TIMESTAMPTZ(6),
    "errors" TEXT,
    PRIMARY KEY ("id")
);
CREATE TABLE "fabriclike_catalogs" (
    "loader" "fabriclike_loader" NOT NULL,
    "games" JSON NOT NULL,
    "groups" JSON NOT NULL,
    "artifacts" JSON NOT NULL,
    PRIMARY KEY ("loader")
);
CREATE TABLE "profiles" (
    "loader" "minecraft_loader" NOT NULL,
    "minecraft_version" TEXT NOT NULL,
    "loader_version" TEXT NOT NULL,
    "metadata" JSON NOT NULL,
    PRIMARY KEY ("loader", "minecraft_version", "loader_version")
);
CREATE INDEX "index_profiles_by_minecraft_version" ON "profiles" ("minecraft_version");
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
