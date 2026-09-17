-- Owyx: launcher telemetry is opt-in for fresh installs.
-- Do NOT edit 20240711194701_init.sql (sqlx checksums break upgrades).
-- Fresh app.db: settings row exists, no instances yet → default telemetry off.
-- Existing users (any instance history) keep their current telemetry value.

UPDATE settings
SET telemetry = 0
WHERE id = 0
  AND telemetry = 1
  AND NOT EXISTS (SELECT 1 FROM instances);
