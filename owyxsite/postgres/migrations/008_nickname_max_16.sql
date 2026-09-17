-- Enforce Minecraft nick length (3–16) for any historical rows that slipped past
-- older 3–32 app validation. Column is already varchar(16) on fresh installs.

-- Truncate overlong nicknames (collision-safe: append _<id> when needed).
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
