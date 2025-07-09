-- This index substantially brings down the cost of the query plan for the
-- hot query at `labrinth::routes::v3::projects::random_projects_get`, from
-- 354.04..363.39 to 171.33..180.68 (~2x improvement).
--
-- The numbers above were calculated in a clean PostgreSQL 17.5.0 container
-- with 10k mods created with the SQL below.
--
-- WITH seq AS (SELECT n FROM GENERATE_SERIES(1, 10000) AS n)
-- INSERT INTO mods (id, team_id, name, summary, icon_url, license_url, slug, status)
-- 	SELECT n, 1, n, '', '', '', n, (ARRAY['approved', 'pending'])[n % 2 + 1] from seq;

CREATE INDEX mods_status ON mods(status);
