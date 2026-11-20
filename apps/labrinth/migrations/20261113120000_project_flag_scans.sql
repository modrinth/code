ALTER TABLE project_flag_run_results
    RENAME COLUMN rules TO rule_results;

ALTER TABLE project_flag_run_results
    ADD COLUMN num_flags BIGINT,
    ADD COLUMN outcome TEXT,
    ADD COLUMN proposed_status TEXT;

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

CREATE TABLE project_flag_rulesets (
    id UUID PRIMARY KEY,
    content_hash TEXT NOT NULL UNIQUE,
    rules JSONB NOT NULL,
    modules JSONB NOT NULL,
    created_by BIGINT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE project_flag_runs
    ADD COLUMN ruleset_id UUID REFERENCES project_flag_rulesets(id);

-- Legacy hashes are namespaced: PostgreSQL cannot compute the canonical Rust
-- BLAKE3 hash. Runtime freezing also compares canonical content to reuse these.
CREATE FUNCTION project_flag_legacy_content_hash(rules JSONB, modules JSONB)
RETURNS TEXT LANGUAGE SQL IMMUTABLE AS $$
    SELECT 'legacy:' || encode(sha256(convert_to(jsonb_build_object(
        'rules', COALESCE((SELECT jsonb_agg(jsonb_build_object(
            'id', rule->'id', 'name', rule->'name', 'params', rule->'params'
        ) ORDER BY (rule->>'id')::BIGINT) FROM jsonb_array_elements(rules) AS rule), '[]'::jsonb),
        'modules', modules
    )::TEXT, 'UTF8')), 'hex');
$$;

INSERT INTO project_flag_rulesets (id, content_hash, rules, modules, created_by, created_at)
SELECT DISTINCT ON (project_flag_legacy_content_hash(snapshots.rules, snapshots.modules))
    snapshots.run_id,
    project_flag_legacy_content_hash(snapshots.rules, snapshots.modules),
    snapshots.rules, snapshots.modules, runs.started_by, runs.started_at
FROM project_flag_run_snapshots AS snapshots
JOIN project_flag_runs AS runs ON runs.id = snapshots.run_id
ORDER BY project_flag_legacy_content_hash(snapshots.rules, snapshots.modules), runs.started_at, snapshots.run_id;

UPDATE project_flag_runs AS runs
SET ruleset_id = rulesets.id
FROM project_flag_run_snapshots AS snapshots, project_flag_rulesets AS rulesets
WHERE snapshots.run_id = runs.id
    AND project_flag_legacy_content_hash(snapshots.rules, snapshots.modules) = rulesets.content_hash;

DROP FUNCTION project_flag_legacy_content_hash(JSONB, JSONB);

ALTER TABLE project_flag_run_snapshots DROP COLUMN rules, DROP COLUMN modules;

CREATE TABLE project_flags_deployed (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    ruleset_id UUID NOT NULL REFERENCES project_flag_rulesets(id),
    deployed_by BIGINT REFERENCES users(id) ON DELETE SET NULL,
    deployed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE FUNCTION reject_project_flag_ruleset_content_update() RETURNS trigger AS $$
BEGIN
    IF NEW.id IS DISTINCT FROM OLD.id
        OR NEW.content_hash IS DISTINCT FROM OLD.content_hash
        OR NEW.rules IS DISTINCT FROM OLD.rules
        OR NEW.modules IS DISTINCT FROM OLD.modules THEN
        RAISE EXCEPTION 'project flag rulesets are immutable';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER project_flag_rulesets_immutable
BEFORE UPDATE ON project_flag_rulesets
FOR EACH ROW EXECUTE FUNCTION reject_project_flag_ruleset_content_update();

CREATE FUNCTION reject_project_flag_run_ruleset_update() RETURNS trigger AS $$
BEGIN
    IF NEW.ruleset_id IS DISTINCT FROM OLD.ruleset_id THEN
        RAISE EXCEPTION 'project flag run ruleset is immutable';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER project_flag_run_ruleset_immutable
BEFORE UPDATE ON project_flag_runs
FOR EACH ROW EXECUTE FUNCTION reject_project_flag_run_ruleset_update();
