DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM mods
        WHERE status NOT IN (
            'approved', 'archived', 'unlisted', 'private', 'draft',
            'processing', 'rejected', 'withheld', 'scheduled'
        )
    ) THEN
        RAISE EXCEPTION 'cannot migrate projects with unknown statuses';
    END IF;

    IF EXISTS (
        SELECT 1 FROM mods
        WHERE status = 'scheduled'
    ) THEN
        RAISE EXCEPTION 'scheduled projects must be resolved before migrating project statuses';
    END IF;
END;
$$;

ALTER TABLE mods
    ADD COLUMN visibility TEXT,
    ADD COLUMN review_status TEXT;

UPDATE mods
SET visibility = CASE
        WHEN status IN ('approved', 'archived') THEN 'listed'
        WHEN status IN ('unlisted', 'private') THEN status
        WHEN requested_status IN ('unlisted', 'private') THEN requested_status
        ELSE 'listed'
    END,
    review_status = CASE
        WHEN status IN ('approved', 'archived', 'unlisted', 'private') THEN 'approved'
        WHEN status = 'processing' THEN 'pending'
        ELSE status
    END;

CREATE TABLE project_history (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL,
    done_by BIGINT,
    occurred_at TIMESTAMPTZ NOT NULL,
    delta JSONB NOT NULL
);

CREATE INDEX project_history_project_id_id ON project_history (project_id, id);


ALTER TABLE mods
    ALTER COLUMN visibility SET NOT NULL,
    ALTER COLUMN review_status SET NOT NULL,
    DROP COLUMN status,
    DROP COLUMN requested_status;

CREATE INDEX mods_review_status ON mods (review_status);

-- (comment copied from 20261117120000_project_visibility_review_status.sql)
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

CREATE INDEX mods_searchable_ids_gist ON mods USING gist (POINT(id, 0))
    WHERE visibility = 'listed' AND review_status = 'approved';
