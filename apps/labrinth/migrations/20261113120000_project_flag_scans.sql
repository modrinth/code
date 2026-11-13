CREATE TABLE project_flag_image_scans (
    image_url TEXT NOT NULL,
    scan_kind TEXT NOT NULL,
    model_version BIGINT NOT NULL CHECK (model_version >= 0),
    probability DOUBLE PRECISION NOT NULL CHECK (probability >= 0 AND probability <= 1),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (image_url, scan_kind, model_version)
);
