CREATE TABLE user_preferences (
	user_id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
	preferences JSONB NOT NULL,
	CONSTRAINT user_preferences_object CHECK (
		jsonb_typeof(preferences) = 'object'
	)
);
