-- Account activity search indexes + launcher telemetry (no PII by design).
-- Site account events stay in user_activity; admin moderation in admin_logs.

CREATE INDEX IF NOT EXISTS idx_user_activity_created_at
  ON user_activity (created_at DESC);

CREATE INDEX IF NOT EXISTS idx_user_activity_type
  ON user_activity (activity_type);

CREATE INDEX IF NOT EXISTS idx_user_activity_user_id_created
  ON user_activity (user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS launcher_telemetry (
  id              BIGSERIAL PRIMARY KEY,
  install_id      UUID NOT NULL,
  user_id         INTEGER REFERENCES users(id) ON DELETE SET NULL,
  event_kind      VARCHAR(40) NOT NULL,
  message         TEXT,
  app_version     VARCHAR(32),
  os_name         VARCHAR(32),
  os_version      VARCHAR(64),
  arch            VARCHAR(16),
  cpu_cores       SMALLINT,
  ram_mb          INTEGER,
  locale          VARCHAR(16),
  metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
  ip_address      VARCHAR(45),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_launcher_telemetry_created
  ON launcher_telemetry (created_at DESC);

CREATE INDEX IF NOT EXISTS idx_launcher_telemetry_kind
  ON launcher_telemetry (event_kind);

CREATE INDEX IF NOT EXISTS idx_launcher_telemetry_install
  ON launcher_telemetry (install_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_launcher_telemetry_user
  ON launcher_telemetry (user_id, created_at DESC)
  WHERE user_id IS NOT NULL;

COMMENT ON TABLE launcher_telemetry IS
  'Anonymous launcher stats/errors. No emails, nicks, or paths with PII.';
