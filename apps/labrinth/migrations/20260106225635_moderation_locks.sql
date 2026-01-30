CREATE TABLE moderation_locks (
	project_id BIGINT PRIMARY KEY REFERENCES mods(id) ON DELETE CASCADE,
	moderator_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	locked_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_moderation_locks_moderator ON moderation_locks(moderator_id);
CREATE INDEX idx_moderation_locks_locked_at ON moderation_locks(locked_at);
