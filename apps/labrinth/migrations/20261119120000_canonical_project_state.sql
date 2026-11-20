ALTER TABLE mods
  ADD COLUMN visibility TEXT NOT NULL DEFAULT 'private',
  ADD COLUMN in_review BOOLEAN NOT NULL DEFAULT FALSE,
  ADD COLUMN review_outcome TEXT NOT NULL DEFAULT 'never_reviewed';

UPDATE mods
SET visibility = CASE
    WHEN status = 'scheduled' THEN 'private'
    WHEN status IN ('approved', 'archived') THEN 'public'
    WHEN status IN ('unlisted', 'withheld') THEN 'unlisted'
    WHEN status = 'private' THEN 'private'
    WHEN requested_status IN ('approved', 'archived') THEN 'public'
    WHEN requested_status = 'unlisted' THEN 'unlisted'
    WHEN requested_status = 'private' THEN 'private'
    WHEN status IN ('processing', 'rejected') THEN 'public'
    ELSE 'private'
  END,
  in_review = status = 'processing',
  review_outcome = CASE
    WHEN status IN ('approved', 'archived', 'unlisted', 'private') THEN 'approved'
    WHEN status = 'rejected' THEN 'rejected'
    WHEN status = 'withheld' THEN 'withheld'
    ELSE 'never_reviewed'
  END;

CREATE TABLE project_history (
  id BIGSERIAL PRIMARY KEY,
  project_id BIGINT NOT NULL,
  done_by BIGINT,
  occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  delta JSONB NOT NULL
);

CREATE INDEX project_history_project_id_occurred_at_idx
  ON project_history (project_id, occurred_at, id);

-- (copied from 20250609134334_spatial-random-project-index.sql)
--
-- The spatial query for retrieving random searchable projects is greatly sped
-- up by this index on a fixture of 1M mods, bringing down the total cost of
-- the query plan and runtime to be comparable to primary key lookups. See the
-- `labrinth::routes::v3::projects::random_projects_get` function and the
-- previous 20250608183828_random-project-index.sql migration for more details.
--
-- That previous migration created a non-spatial index for the status column which
-- does not get used in the new spatial query, but may still be useful for other
-- queries that filter mods by status.

CREATE INDEX mods_searchable_visibility_ids_gist ON mods USING gist (POINT(id, 0))
  WHERE visibility = 'public' AND review_outcome = 'approved';

-- DROP INDEX IF EXISTS mods_searchable_ids_gist;
-- DROP INDEX IF EXISTS mods_status;
-- ALTER TABLE mods DROP COLUMN status, DROP COLUMN requested_status;
