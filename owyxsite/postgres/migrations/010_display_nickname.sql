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

ALTER TABLE users ALTER COLUMN display_nickname SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS users_display_nickname_lower_uidx
  ON users (LOWER(display_nickname));
