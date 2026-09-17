-- Allow duplicate display nicknames (play / in-game labels).
-- Login (`nickname`) and email stay unique.

DROP INDEX IF EXISTS users_display_nickname_lower_uidx;

DROP TRIGGER IF EXISTS users_identity_cross_unique_trg ON users;
DROP FUNCTION IF EXISTS users_identity_cross_unique();
