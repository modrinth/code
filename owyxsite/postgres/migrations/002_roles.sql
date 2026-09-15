-- 002_roles.sql — normalize user roles for the redesigned Admin role UI.
-- Idempotent and non-destructive. Safe to re-run. Restore path: old.backup / init.sql.
--
-- Roles used by the app/API (owyxsite/backend/src/routes/auth.js requireRole,
-- admin PUT /users/:id/role): user | helper | moderator | admin.
-- Default on registration stays 'user' (users.role DEFAULT 'user').

BEGIN;

-- 1) Backfill any NULL/empty roles to the default so the CHECK below is safe.
UPDATE public.users SET role = 'user' WHERE role IS NULL OR btrim(role) = '';

-- 2) Fold any legacy/unknown role value into 'user' so the constraint can apply
--    without failing on pre-existing data. Adjust here if you add real roles.
UPDATE public.users
SET role = 'user'
WHERE role NOT IN ('user', 'helper', 'moderator', 'admin');

-- 3) Add a CHECK constraint on allowed roles, only if it isn't there yet.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'users_role_allowed'
    ) THEN
        ALTER TABLE public.users
            ADD CONSTRAINT users_role_allowed
            CHECK (role IN ('user', 'helper', 'moderator', 'admin'));
    END IF;
END$$;

-- 4) Keep the sane default explicit.
ALTER TABLE public.users ALTER COLUMN role SET DEFAULT 'user';

COMMIT;
