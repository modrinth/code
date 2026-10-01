CREATE TABLE "download_blobs" (
    "download_run_id" UUID NOT NULL,
    "url" TEXT NOT NULL,
    "sha256" TEXT NOT NULL,
    PRIMARY KEY ("download_run_id", "url")
);
CREATE INDEX "index_download_blobs_by_sha256" ON "download_blobs" ("sha256");
CREATE TABLE "download_runs" (
    "id" UUID NOT NULL,
    "started_at" TIMESTAMPTZ(6) NOT NULL,
    "completed_at" TIMESTAMPTZ(6),
    "errors" TEXT,
    PRIMARY KEY ("id")
);
CREATE TABLE "json_blobs" (
    "sha256" TEXT NOT NULL,
    "json" TEXT NOT NULL,
    PRIMARY KEY ("sha256")
);
