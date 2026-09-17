ALTER TABLE user_limits
	ALTER COLUMN versions_per_day SET DEFAULT 150;

-- rows that existed before 20260916120000 were backfilled with the old default
UPDATE user_limits
	SET versions_per_day = 150
	WHERE versions_per_day = 50;
