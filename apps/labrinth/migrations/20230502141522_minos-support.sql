-- No longer have banned users in Labrinth
DROP TABLE banned_users;

-- Initialize kratos_id 
ALTER TABLE users ADD COLUMN kratos_id varchar(40) UNIQUE;

-- Add pats table
CREATE TABLE pats (
    id BIGINT PRIMARY KEY,
    name VARCHAR(255),
    user_id BIGINT NOT NULL REFERENCES users(id),
    access_token VARCHAR(64) NOT NULL,
    scope BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL
);