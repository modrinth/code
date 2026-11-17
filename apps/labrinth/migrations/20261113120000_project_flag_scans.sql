ALTER TABLE project_flag_run_results
    RENAME COLUMN rules TO rule_results;

ALTER TABLE project_flag_run_results
    ADD COLUMN num_flags BIGINT,
    ADD COLUMN outcome TEXT;

UPDATE project_flag_run_results AS results
SET num_flags = (
        SELECT COALESCE(SUM(jsonb_array_length(rule.value->'flags')), 0)::BIGINT
        FROM jsonb_each(results.rule_results) AS rule
    ),
    outcome = CASE
        WHEN results.error IS NOT NULL OR EXISTS (
            SELECT 1 FROM jsonb_each(results.rule_results) AS rule
            WHERE rule.value->>'error' IS NOT NULL
        ) THEN 'error'
        WHEN EXISTS (
            SELECT 1 FROM jsonb_each(results.rule_results) AS rule
            WHERE jsonb_array_length(rule.value->'flags') > 0
        ) THEN 'flagged'
        ELSE 'no_flags'
    END;

ALTER TABLE project_flag_run_results
    ALTER COLUMN num_flags SET NOT NULL,
    ADD CHECK (num_flags >= 0),
    ALTER COLUMN outcome SET NOT NULL,
    ADD CHECK (outcome IN ('flagged', 'no_flags', 'error'));

DROP INDEX project_flag_run_results_errors;

CREATE INDEX project_flag_run_results_run_id_outcome
    ON project_flag_run_results (run_id, outcome) INCLUDE (num_flags);

CREATE TABLE project_flag_modules (
    name VARCHAR(1024) PRIMARY KEY,
    script TEXT NOT NULL
);

CREATE TABLE project_flag_scans (
    scan_kind TEXT NOT NULL,
    input_hash BYTEA NOT NULL,
    model_version BIGINT NOT NULL CHECK (model_version >= 0),
    result JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (scan_kind, input_hash, model_version)
);

CREATE TABLE project_flag_run_snapshots (
    run_id UUID PRIMARY KEY REFERENCES project_flag_runs(id) ON DELETE CASCADE,
    rules JSONB NOT NULL,
    modules JSONB NOT NULL,
    project_ids BIGINT[] NOT NULL
);

-- Legacy runs have no durable rule/module snapshot. Preserve their results and
-- explicitly report incomplete work rather than evaluating with different rules.
UPDATE project_flag_runs
SET status = 'done', completed_at = CURRENT_TIMESTAMP,
    error = 'durable run snapshot unavailable after migration; remaining projects were not evaluated'
WHERE status = 'running';

DROP TABLE project_flag_run_pending_jobs;

-- Results must survive project deletion so snapshot progress remains accurate.
ALTER TABLE project_flag_run_results
    DROP CONSTRAINT project_flag_run_results_project_id_fkey;
