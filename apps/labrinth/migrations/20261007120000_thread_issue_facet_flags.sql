ALTER TABLE threads_issue_facets
	ADD COLUMN user_addressed BOOLEAN NOT NULL DEFAULT FALSE,
	ADD COLUMN moderator_verified BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE threads_issue_facets facet
SET
	user_addressed = issue.user_addressed,
	moderator_verified = issue.moderator_verified
FROM threads_issues issue
WHERE facet.issue_id = issue.id;

ALTER TABLE threads_issues
	DROP COLUMN user_addressed,
	DROP COLUMN moderator_verified;
