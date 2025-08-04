-- Add migration script here

CREATE TABLE IF NOT EXISTS users_redeemals (
    id         SERIAL PRIMARY KEY,
    user_id    BIGINT NOT NULL REFERENCES users(id),
    offer      VARCHAR NOT NULL,
    redeemed   TIMESTAMP WITH TIME ZONE NOT NULL,
    status     VARCHAR NOT NULL
);