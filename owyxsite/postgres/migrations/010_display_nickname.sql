-- Display nickname (visible / in-game) separate from login nickname.
-- Also email_changed_at for 30-day email change cooldown.

ALTER TABLE users ADD COLUMN IF NOT EXISTS display_nickname VARCHAR(16);
ALTER TABLE users ADD COLUMN IF NOT EXISTS display_nickname_changed_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_changed_at TIMESTAMPTZ;

UPDATE users
SET display_nickname = LEFT(nickname, 16)
WHERE display_nickname IS NULL AND nickname IS NOT NULL;

UPDATE users
SET display_nickname = LEFT(COALESCE(nickname, 'Player'), 16)
WHERE display_nickname IS NULL;

-- Collision-safe dedup for case-insensitive display nicknames (same pattern as 008/009).
DO $$
DECLARE
  r RECORD;
  candidate TEXT;
BEGIN
  FOR r IN
    SELECT id, display_nickname
    FROM users
    WHERE display_nickname IS NOT NULL
    ORDER BY id
  LOOP
    candidate := left(r.display_nickname, 16);
    IF EXISTS (
      SELECT 1 FROM users u
      WHERE u.id < r.id
        AND lower(COALESCE(u.display_nickname, u.nickname)) = lower(candidate)
    ) THEN
      candidate := left(r.display_nickname, greatest(1, 16 - 1 - length(r.id::text)))
                  || '_' || r.id::text;
      candidate := left(candidate, 16);
      UPDATE users SET display_nickname = candidate WHERE id = r.id;
    END IF;
  END LOOP;
END $$;

-- Collision-safe dedup for case-insensitive login nicknames before lower unique index.
DO $$
DECLARE
  r RECORD;
  candidate TEXT;
BEGIN
  FOR r IN
    SELECT id, nickname FROM users WHERE nickname IS NOT NULL ORDER BY id
  LOOP
    candidate := left(r.nickname, 16);
    IF EXISTS (
      SELECT 1 FROM users u
      WHERE u.id < r.id AND lower(u.nickname) = lower(candidate)
    ) THEN
      candidate := left(r.nickname, greatest(1, 16 - 1 - length(r.id::text)))
                  || '_' || r.id::text;
      candidate := left(candidate, 16);
      UPDATE users SET nickname = candidate WHERE id = r.id;
    END IF;
  END LOOP;
END $$;

ALTER TABLE users ALTER COLUMN display_nickname SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS users_display_nickname_lower_uidx
  ON users (LOWER(display_nickname));

CREATE UNIQUE INDEX IF NOT EXISTS users_nickname_lower_uidx
  ON users (LOWER(nickname));

-- Cross-field: display must not collide with another user's login (and vice versa).
CREATE OR REPLACE FUNCTION users_identity_cross_unique()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM users u
    WHERE u.id IS DISTINCT FROM NEW.id
      AND (
        LOWER(u.nickname) = LOWER(NEW.display_nickname)
        OR LOWER(COALESCE(u.display_nickname, u.nickname)) = LOWER(NEW.nickname)
      )
  ) THEN
    RAISE EXCEPTION 'identity collision between nickname and display_nickname'
      USING ERRCODE = 'unique_violation';
  END IF;
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS users_identity_cross_unique_trg ON users;
CREATE TRIGGER users_identity_cross_unique_trg
  BEFORE INSERT OR UPDATE OF nickname, display_nickname ON users
  FOR EACH ROW
  EXECUTE FUNCTION users_identity_cross_unique();
