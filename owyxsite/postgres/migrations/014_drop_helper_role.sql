-- Drop legacy "helper" role. Moderators get read-only admin UI + full logs.
-- Existing helper accounts become regular users.

UPDATE users SET role = 'user' WHERE role = 'helper';

ALTER TABLE users DROP CONSTRAINT IF EXISTS users_role_check;

ALTER TABLE users
  ADD CONSTRAINT users_role_check
  CHECK (role IN ('user', 'moderator', 'admin'));
