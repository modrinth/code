ALTER TABLE collections
    ADD COLUMN follows integer NOT NULL DEFAULT 0;

CREATE TABLE collection_follows (
    follower_id bigint REFERENCES users NOT NULL,
    collection_id bigint REFERENCES collections NOT NULL,
    created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (follower_id, collection_id)
);

CREATE INDEX collection_follows_collection_id_idx
    ON collection_follows (collection_id);
