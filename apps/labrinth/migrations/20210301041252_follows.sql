CREATE TABLE mod_follows(
    follower_id bigint REFERENCES users NOT NULL,
    mod_id bigint REFERENCES mods NOT NULL,
    created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (follower_id, mod_id)
);

ALTER TABLE mods
    ADD COLUMN follows integer NOT NULL default 0;