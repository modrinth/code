CREATE TABLE blocked_users (
    user_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    blocked_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, blocked_id)
);
