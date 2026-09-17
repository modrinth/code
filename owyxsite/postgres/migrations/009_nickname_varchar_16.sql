-- Idempotent: shrink users.nickname to varchar(16) on DBs that already ran
-- an earlier 008 (truncate-only) without ALTER COLUMN.
-- Safe to re-run when the column is already varchar(16).

DO $$
DECLARE
  r RECORD;
  candidate TEXT;
BEGIN
  FOR r IN
    SELECT id, nickname FROM users WHERE char_length(nickname) > 16
  LOOP
    candidate := left(r.nickname, 16);
    IF EXISTS (
      SELECT 1 FROM users u
      WHERE lower(u.nickname) = lower(candidate) AND u.id <> r.id
    ) THEN
      candidate := left(r.nickname, greatest(1, 16 - 1 - length(r.id::text))) || '_' || r.id::text;
      candidate := left(candidate, 16);
    END IF;
    UPDATE users SET nickname = candidate WHERE id = r.id;
  END LOOP;
END $$;

ALTER TABLE users ALTER COLUMN nickname TYPE character varying(16);
