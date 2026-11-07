ALTER TABLE project_flag_runs
    ADD COLUMN applied_by BIGINT REFERENCES users(id) ON DELETE SET NULL;
