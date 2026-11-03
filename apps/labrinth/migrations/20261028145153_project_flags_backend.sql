CREATE TABLE project_flag_rules (
  id         BIGINT         PRIMARY KEY,
  name       VARCHAR(1023)  NOT NULL,
  created_by BIGINT         REFERENCES users(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ    NOT NULL,
  updated_by BIGINT         REFERENCES users(id) ON DELETE SET NULL,
  updated_at TIMESTAMPTZ    NOT NULL,
  params     JSONB          NOT NULL
);

CREATE TABLE project_flags (
  id      BIGINT PRIMARY KEY,
  rule_id BIGINT REFERENCES project_flag_rules(id) ON DELETE SET NULL,
  what    JSONB  NOT NULL
);

CREATE TABLE project_flag_runs (
  id             UUID        PRIMARY KEY,
  status         TEXT        NOT NULL,
  total_projects BIGINT      NOT NULL,
  started_by     BIGINT      REFERENCES users(id) ON DELETE SET NULL,
  started_at     TIMESTAMPTZ NOT NULL,
  completed_at   TIMESTAMPTZ,
  output         JSONB,
  error          TEXT
);

CREATE TABLE project_flag_run_results (
  run_id       UUID        NOT NULL REFERENCES project_flag_runs(id) ON DELETE CASCADE,
  project_id   BIGINT      NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
  rules        JSONB       NOT NULL DEFAULT '{}'::jsonb,
  error        TEXT,
  evaluated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (run_id, project_id)
);

CREATE INDEX project_flag_run_results_errors
  ON project_flag_run_results (run_id)
  WHERE error IS NOT NULL;
