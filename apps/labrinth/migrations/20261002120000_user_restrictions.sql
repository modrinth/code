CREATE TABLE user_restrictions (
	user_id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
	removed_perms BIGINT NOT NULL,
	reason TEXT,
	private_reason TEXT,
	restricted_by BIGINT NOT NULL REFERENCES users(id),
	updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
