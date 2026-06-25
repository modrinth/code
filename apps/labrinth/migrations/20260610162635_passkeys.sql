CREATE TABLE user_passkeys (
    id 				BIGINT PRIMARY KEY,
    user_id 		BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name 			VARCHAR(255) NOT NULL,
    credential_id	BYTEA NOT NULL UNIQUE,
    passkey         JSONB NOT NULL,
    created_at		TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used		TIMESTAMPTZ
);

CREATE INDEX user_passkeys_user_id ON user_passkeys (user_id);
