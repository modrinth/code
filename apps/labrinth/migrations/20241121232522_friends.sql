CREATE TABLE friends (
    user_id BIGINT NOT NULL REFERENCES users(id),
    friend_id BIGINT NOT NULL REFERENCES users(id),
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    accepted BOOLEAN NOT NULL DEFAULT FALSE,

    PRIMARY KEY (user_id, friend_id)
);

ALTER TABLE users
    ADD COLUMN allow_friend_requests BOOLEAN NOT NULL DEFAULT TRUE;
