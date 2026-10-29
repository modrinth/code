CREATE TABLE project_flag_rules (
  id         BIGINT         PRIMARY KEY,
  name       VARCHAR(1023)  NOT NULL,
  created_by BIGINT         REFERENCES users(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ    NOT NULL,
  updated_by BIGINT         REFERENCES users(id) ON DELETE SET NULL,
  updated_at TIMESTAMPTZ    NOT NULL,
  expression VARCHAR(65535) NOT NULL
);

CREATE TABLE project_flags (
  id      BIGINT PRIMARY KEY,
  rule_id BIGINT REFERENCES project_flag_rules(id) ON DELETE SET NULL,
  what    JSONB  NOT NULL
);

CREATE TABLE project_flag_runs (
  id           BIGINT      PRIMARY KEY,
  status       TEXT        NOT NULL,
  started_by   BIGINT      REFERENCES users(id) ON DELETE SET NULL,
  started_at   TIMESTAMPTZ NOT NULL,
  completed_at TIMESTAMPTZ,
  output       JSONB,
  error        TEXT
);
