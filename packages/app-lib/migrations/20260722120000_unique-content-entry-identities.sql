-- Older app versions could save the same content entry more than once. Remove the older copies before preventing future duplicates.
WITH ranked_entries AS (
	SELECT
		id,
		ROW_NUMBER() OVER (
			PARTITION BY content_set_id, file_id
			ORDER BY modified_at DESC, added_at DESC, id DESC
		) AS duplicate_rank
	FROM instance_content_entries
	WHERE file_id IS NOT NULL
)
DELETE FROM instance_content_entries
WHERE id IN (
	SELECT id
	FROM ranked_entries
	WHERE duplicate_rank > 1
);

WITH ranked_entries AS (
	SELECT
		id,
		ROW_NUMBER() OVER (
			PARTITION BY content_set_id, project_id, version_id
			ORDER BY modified_at DESC, added_at DESC, id DESC
		) AS duplicate_rank
	FROM instance_content_entries
	WHERE file_id IS NULL
		AND project_id IS NOT NULL
		AND version_id IS NOT NULL
)
DELETE FROM instance_content_entries
WHERE id IN (
	SELECT id
	FROM ranked_entries
	WHERE duplicate_rank > 1
);

CREATE UNIQUE INDEX instance_content_entries_content_set_file_unique
	ON instance_content_entries(content_set_id, file_id)
	WHERE file_id IS NOT NULL;

CREATE UNIQUE INDEX instance_content_entries_content_set_project_version_unique
	ON instance_content_entries(content_set_id, project_id, version_id)
	WHERE file_id IS NULL
		AND project_id IS NOT NULL
		AND version_id IS NOT NULL;
