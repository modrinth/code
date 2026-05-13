CREATE TABLE moderation_notes (
	user_id BIGINT NULL REFERENCES users(id) ON DELETE CASCADE,
	organization_id BIGINT NULL REFERENCES organizations(id) ON DELETE CASCADE,
	last_modified TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	last_author BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	version INTEGER NOT NULL DEFAULT 0,
	notes TEXT NOT NULL,
	user_rating INTEGER NOT NULL DEFAULT 0,
	CONSTRAINT moderation_notes_one_target CHECK (
		(user_id IS NOT NULL AND organization_id IS NULL)
		OR (user_id IS NULL AND organization_id IS NOT NULL)
	)
);

CREATE UNIQUE INDEX moderation_notes_user_id_unique
	ON moderation_notes(user_id)
	WHERE user_id IS NOT NULL;

CREATE UNIQUE INDEX moderation_notes_organization_id_unique
	ON moderation_notes(organization_id)
	WHERE organization_id IS NOT NULL;

CREATE INDEX moderation_notes_user_id_idx ON moderation_notes(user_id);
CREATE INDEX moderation_notes_organization_id_idx ON moderation_notes(organization_id);
