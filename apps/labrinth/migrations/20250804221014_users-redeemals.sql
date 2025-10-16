CREATE TABLE users_redeemals (
    id            SERIAL PRIMARY KEY,
    user_id       BIGINT NOT NULL REFERENCES users(id),
    offer         VARCHAR NOT NULL,
    redeemed      TIMESTAMP WITH TIME ZONE NOT NULL,
    status        VARCHAR NOT NULL,
    last_attempt  TIMESTAMP WITH TIME ZONE,
    n_attempts    INTEGER NOT NULL
);