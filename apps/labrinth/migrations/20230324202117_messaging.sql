-- Add migration script here

-- Add route for users to see their own reports

CREATE TABLE threads (
    id bigint PRIMARY KEY,
    -- can be either "report", "project", or "direct_message". direct message is unused for now
    thread_type VARCHAR(64) NOT NULL
);

CREATE TABLE threads_messages (
    id bigint PRIMARY KEY,
    thread_id bigint REFERENCES threads ON UPDATE CASCADE NOT NULL,
    -- If this is null, it's a system message
    author_id bigint REFERENCES users ON UPDATE CASCADE NULL,
    body jsonb NOT NULL,
    created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    show_in_mod_inbox BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE threads_members (
 thread_id bigint REFERENCES threads ON UPDATE CASCADE NOT NULL,
 user_id bigint REFERENCES users ON UPDATE CASCADE NOT NULL,
 PRIMARY KEY (thread_id, user_id)
);

ALTER TABLE reports
    ADD COLUMN closed boolean NOT NULL DEFAULT FALSE;
ALTER TABLE reports
    ADD COLUMN thread_id bigint references threads ON UPDATE CASCADE;
ALTER TABLE mods
    ADD COLUMN thread_id bigint references threads ON UPDATE CASCADE;
