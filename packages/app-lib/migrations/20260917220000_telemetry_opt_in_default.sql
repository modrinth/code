-- Owyx: launcher telemetry is opt-in (default off for fresh installs).
-- init.sql now uses DEFAULT FALSE for settings.telemetry.
-- Do NOT mass-update existing rows — users who already have telemetry=1 stay opted in.
-- SQLite cannot ALTER COLUMN DEFAULT in-place; this migration documents the policy
-- for databases created before the init.sql change (their DEFAULT remains historical,
-- but the only settings INSERT is on first boot and already ran).

SELECT 1 WHERE 0;
