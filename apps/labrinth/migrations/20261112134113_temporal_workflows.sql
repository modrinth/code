CREATE TABLE workflow_memos (
  activity TEXT NOT NULL,
  key BYTEA NOT NULL,
  out JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (activity, key)
);
