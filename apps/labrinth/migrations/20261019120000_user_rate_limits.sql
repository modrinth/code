ALTER TABLE user_limits
	ADD COLUMN requests_per_minute INTEGER
		CHECK (requests_per_minute > 0);
