-- Email-change flow: a pending new email + a short-lived confirmation code sent
-- to the NEW address. Idempotent.
--   psql -U owyx_user -d owyx_db -f owyxsite/postgres/migrations/005_email_change.sql

ALTER TABLE public.users ADD COLUMN IF NOT EXISTS pending_email character varying(255);
ALTER TABLE public.users ADD COLUMN IF NOT EXISTS email_change_code character varying(12);
ALTER TABLE public.users ADD COLUMN IF NOT EXISTS email_change_expires timestamp without time zone;
-- Wrong-code attempts for the current pending change (rate limit); reset per request.
ALTER TABLE public.users ADD COLUMN IF NOT EXISTS email_change_attempts integer DEFAULT 0;
