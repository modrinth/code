-- Track when a user last changed their nickname, to enforce a 30-day cooldown.
-- Idempotent: safe to run on an already-migrated database.
--   psql -U owyx_user -d owyx_db -f owyxsite/postgres/migrations/004_nickname_cooldown.sql

ALTER TABLE public.users ADD COLUMN IF NOT EXISTS nickname_changed_at timestamp without time zone;
