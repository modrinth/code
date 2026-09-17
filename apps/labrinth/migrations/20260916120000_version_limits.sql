ALTER TABLE user_limits
	ADD COLUMN projects_per_day INTEGER NOT NULL DEFAULT 10,
	ADD COLUMN versions_per_project INTEGER NOT NULL DEFAULT 10000,
	ADD COLUMN versions_per_day INTEGER NOT NULL DEFAULT 50,
	ADD CONSTRAINT user_limits_user_id_unique
		UNIQUE NULLS NOT DISTINCT (user_id);

CREATE INDEX versions_author_id_date_published
	ON versions (author_id, date_published);
