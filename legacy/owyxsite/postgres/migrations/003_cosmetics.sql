-- Cosmetics foundation for Owyx accounts (skins, capes).
-- Idempotent: safe to run on an already-migrated database.
--   psql -U owyx_user -d owyx_db -f owyxsite/postgres/migrations/003_cosmetics.sql
--
-- The launcher reads skin_url / skin_model from GET /api/launcher/me and applies
-- the account skin. Capes are reserved for a later stage (column added now so the
-- contract is stable).

ALTER TABLE public.users ADD COLUMN IF NOT EXISTS skin_url character varying(255);
ALTER TABLE public.users ADD COLUMN IF NOT EXISTS skin_model character varying(10) DEFAULT 'classic';
ALTER TABLE public.users ADD COLUMN IF NOT EXISTS cape_url character varying(255);
ALTER TABLE public.users ADD COLUMN IF NOT EXISTS cosmetics_updated_at timestamp without time zone;

-- Keep skin_model constrained to the two Minecraft body models.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'users_skin_model_check'
          AND table_name = 'users'
    ) THEN
        ALTER TABLE public.users
            ADD CONSTRAINT users_skin_model_check
            CHECK (skin_model IN ('classic', 'slim'));
    END IF;
END $$;
