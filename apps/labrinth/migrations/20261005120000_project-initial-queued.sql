ALTER TABLE mods ADD COLUMN initial_queued timestamptz;

WITH last_approval AS (
	SELECT t.mod_id, MAX(tm.created) AS created
	FROM threads t
	INNER JOIN threads_messages tm ON tm.thread_id = t.id
	WHERE tm.body ->> 'type' = 'status_change'
		AND tm.body ->> 'new_status' IN ('approved', 'archived', 'unlisted', 'private')
	GROUP BY t.mod_id
),
first_submission AS (
	SELECT t.mod_id, MIN(tm.created) AS created
	FROM threads t
	INNER JOIN threads_messages tm ON tm.thread_id = t.id
	LEFT JOIN last_approval la ON la.mod_id = t.mod_id
	WHERE tm.body ->> 'type' = 'status_change'
		AND tm.body ->> 'new_status' = 'processing'
		AND tm.created > COALESCE(la.created, '-infinity')
	GROUP BY t.mod_id
)
UPDATE mods m
SET initial_queued = first_submission.created
FROM first_submission
WHERE first_submission.mod_id = m.id
	AND m.status NOT IN ('approved', 'archived', 'unlisted', 'private');

UPDATE mods
SET initial_queued = queued
WHERE status = 'processing'
	AND initial_queued IS NULL;
