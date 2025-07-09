-- The spatial query for retrieving random searchable projects is greatly sped
-- up by this index on a fixture of 1M mods, bringing down the total cost of
-- the query plan and runtime to be comparable to primary key lookups. See the
-- `labrinth::routes::v3::projects::random_projects_get` function and the
-- previous 20250608183828_random-project-index.sql migration for more details.
--
-- That previous migration created a non-spatial index for the status column which
-- does not get used in the new spatial query, but may still be useful for other
-- queries that filter mods by status.

CREATE INDEX mods_searchable_ids_gist ON mods USING gist (POINT(id, 0))
    WHERE status = ANY(ARRAY['approved', 'archived']);
