CREATE TABLE moderation_flag_rules (
  id         BIGINT         NOT NULL,
  name       VARCHAR(1023)  NOT NULL,
  created_by BIGINT         REFERENCES users(id) ON DELETE UPDATE,
  created_at TIMESTAMPTZ    NOT NULL,
  updated_by BIGINT         REFERENCES users(id) ON DELETE UPDATE,
  updated_at TIMESTAMPTZ    NOT NULL,
  expression VARCHAR(65535) NOT NULL
);

CREATE TABLE moderation_flags (
  id      BIGINT NOT NULL,
  rule_id BIGINT REFERENCES moderation_flag_rules(id) ON DELETE UPDATE,
  what    JSONB  NOT NULL
);

CREATE TABLE moderation_flag_runs (
  id           BIGINT      NOT NULL,
  status       TEXT        NOT NULL,
  started_by   BIGINT      REFERENCES users(id) ON DELETE UPDATE,
  started_at   TIMESTAMPTZ NOT NULL,
  completed_at TIMESTAMPTZ,
  output       JSONB,
  error        TEXT
);
