CREATE TABLE install_job_requests (
	job_id TEXT PRIMARY KEY NOT NULL REFERENCES install_jobs(id) ON DELETE CASCADE,
	request JSONB NOT NULL
);

INSERT INTO install_job_requests (job_id, request)
SELECT id, json_extract(state, '$.request')
FROM install_jobs
WHERE status NOT IN ('succeeded', 'canceled')
	AND json_extract(state, '$.request') IS NOT NULL;

UPDATE install_jobs
SET state = json_set(
	state,
	'$.content_count',
	json_array_length(state, '$.request.updates')
)
WHERE json_extract(state, '$.request.type') = 'bulk_update_content';

UPDATE install_jobs
SET state = json_remove(state, '$.request');
