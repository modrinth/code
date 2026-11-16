CREATE TABLE project_flag_run_pending_jobs (
    run_id UUID NOT NULL REFERENCES project_flag_runs(id) ON DELETE CASCADE,
    project_id BIGINT NOT NULL,
    PRIMARY KEY (run_id, project_id)
);
