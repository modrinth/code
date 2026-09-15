-- Servers + packs catalog (site control-plane for the launcher).
-- Idempotent: safe to run repeatedly.
--   docker compose exec -T postgres psql -U owyx_user -d owyx_db < postgres/migrations/007_servers_packs.sql
--
-- Existing Docker volumes do NOT re-run initdb.d. Apply this file by hand
-- on any database that was created before this migration existed.

CREATE TABLE IF NOT EXISTS public.packs (
    id TEXT PRIMARY KEY,
    name VARCHAR(120) NOT NULL,
    minecraft VARCHAR(32) NOT NULL,
    loader VARCHAR(32) NOT NULL DEFAULT 'vanilla',
    icon_url TEXT,
    description TEXT NOT NULL DEFAULT '',
    source_type VARCHAR(32) NOT NULL DEFAULT 'http_zip',
    source_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    manifest_url TEXT,
    published BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_by INTEGER
);

CREATE TABLE IF NOT EXISTS public.servers (
    id TEXT PRIMARY KEY,
    name VARCHAR(120) NOT NULL,
    icon_url TEXT,
    address VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL DEFAULT 25565,
    kind VARCHAR(32) NOT NULL DEFAULT 'owyx',
    pack_id TEXT REFERENCES public.packs(id) ON DELETE SET NULL,
    minecraft VARCHAR(32),
    loader VARCHAR(32),
    requires_account BOOLEAN NOT NULL DEFAULT false,
    published BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_by INTEGER
);

CREATE INDEX IF NOT EXISTS packs_published_idx ON public.packs (published);
CREATE INDEX IF NOT EXISTS servers_published_sort_idx ON public.servers (published, sort_order, name);

-- Friends-sprint demo: one pack (tiny local zip) + two servers sharing it.
-- Relative URLs are resolved to the request host by the API (dev :3001 / prod).
INSERT INTO public.packs (
    id, name, minecraft, loader, description, source_type, source_config, manifest_url, published
)
SELECT
    'demo-vanilla',
    'Owyx Demo Vanilla 1.21.1',
    '1.21.1',
    'vanilla',
    'Крошечный тестовый оверлей для друзей. Лаунчер ставит ваниль, затем распаковывает этот zip в game/.',
    'http_zip',
    '{"url":"/fixtures/packs/demo-vanilla.zip","sha256":"b5b345b133be7e851e47ef6f0fd83c61be3abc56dfc8cacd0c6fc65790d2eadb"}'::jsonb,
    '/api/launcher/v1/packs/demo-vanilla/manifest',
    true
WHERE NOT EXISTS (SELECT 1 FROM public.packs WHERE id = 'demo-vanilla');

INSERT INTO public.servers (
    id, name, address, port, kind, pack_id, minecraft, loader, requires_account, published, sort_order
)
SELECT * FROM (VALUES
    ('owyx-demo', 'Owyx — демо', 'play.owyx.site', 25565, 'owyx', 'demo-vanilla', '1.21.1', 'vanilla', false, true, 0),
    ('owyx-friends', 'Owyx — друзья', '127.0.0.1', 25565, 'community', 'demo-vanilla', '1.21.1', 'vanilla', false, true, 10)
) AS seed(id, name, address, port, kind, pack_id, minecraft, loader, requires_account, published, sort_order)
WHERE NOT EXISTS (SELECT 1 FROM public.servers WHERE id = seed.id);
