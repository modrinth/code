CREATE TABLE project_flag_scans (
    scan_kind TEXT NOT NULL,
    input_hash BYTEA NOT NULL,
    model_version BIGINT NOT NULL CHECK (model_version >= 0),
    result JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (scan_kind, input_hash, model_version)
);
