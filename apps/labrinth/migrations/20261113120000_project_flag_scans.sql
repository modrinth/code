ALTER TABLE project_flags
    DROP CONSTRAINT project_flags_run_id_fkey;

DELETE FROM project_flag_run_results;
DELETE FROM project_flag_runs;

ALTER TABLE project_flag_run_results
    RENAME COLUMN rules TO rule_results;

ALTER TABLE project_flag_run_results
    ADD COLUMN num_flags BIGINT NOT NULL CHECK (num_flags >= 0),
    ADD COLUMN proposed_review_outcome TEXT,
    ADD COLUMN effective_flags JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN skipped BOOLEAN NOT NULL DEFAULT FALSE;

-- reusable modules for rule scripts

CREATE TABLE project_flag_modules (
    name VARCHAR(1024) PRIMARY KEY,
    script TEXT NOT NULL
);

-- cached info from reconciliation workflow activities,
-- e.g. "are the gallery images AI generated"

CREATE TABLE project_flag_scans (
    scan_kind TEXT NOT NULL,
    input_hash BYTEA NOT NULL,
    model_version BIGINT NOT NULL CHECK (model_version >= 0),
    result JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (scan_kind, input_hash, model_version)
);

DROP TABLE project_flag_run_pending_jobs;

ALTER TABLE project_flag_run_results
    DROP CONSTRAINT project_flag_run_results_run_id_fkey;

DROP TABLE project_flag_runs;

-- Results must survive project deletion so snapshot progress remains accurate.
ALTER TABLE project_flag_run_results
    DROP CONSTRAINT project_flag_run_results_project_id_fkey;

CREATE TABLE project_flag_rulesets (
    id UUID PRIMARY KEY,
    content_hash TEXT NOT NULL UNIQUE,
    rules JSONB NOT NULL,
    modules JSONB NOT NULL,
    created_by BIGINT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE project_flags_deployed (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    ruleset_id UUID NOT NULL REFERENCES project_flag_rulesets(id),
    deployed_by BIGINT REFERENCES users(id) ON DELETE SET NULL,
    deployed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
