-- These indices substantially bring down the cost of the query plan for the
-- hot query at `labrinth::routes::v3::projects::random_projects_get`, from
-- ~235 to 0.29..18.92 (~10x improvement; for comparison, a lookup by primary
-- key has 0.29..8.3 cost).
--
-- A clean PostgreSQL 17.5.0 container with 10k mods created with the SQL
-- below seems to only benefit from the `mods_id_md5` index, but the remaining
-- ones cover us in case the query plan changes in the future without undue
-- performance cost on inserts.
--
-- WITH seq AS (SELECT n FROM GENERATE_SERIES(1, 10000) AS n)
-- INSERT INTO mods (id, team_id, name, summary, icon_url, license_url, slug, status)
-- 	SELECT n, 1, n, '', '', '', n, (ARRAY['approved', 'pending'])[n % 2 + 1] from seq;

CREATE INDEX mods_id_md5 ON mods(md5(id::varchar));
CREATE INDEX mods_status ON mods(status);
CREATE INDEX mods_status_and_id_md5 ON mods(status, md5(id::varchar));
