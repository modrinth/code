CREATE TYPE "fabriclike_loader" AS ENUM ('fabric', 'quilt');
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
CREATE TABLE "fabriclike_catalogs" (
    "loader" "fabriclike_loader" NOT NULL,
    "games" JSON NOT NULL,
    "groups" JSON NOT NULL,
    "artifacts" JSON NOT NULL,
    PRIMARY KEY ("loader")
);
