INSERT INTO statuses (status) VALUES ('archived');

ALTER TABLE notifications
    ADD COLUMN type varchar(256);

ALTER TABLE mods
    ADD COLUMN rejection_reason varchar(2000),
    ADD COLUMN rejection_body varchar(65536);

DROP TABLE dependencies;

CREATE TABLE dependencies (
    id serial PRIMARY KEY,
    dependent_id bigint REFERENCES versions ON UPDATE CASCADE NOT NULL,
    dependency_type varchar(255) NOT NULL,
    dependency_id bigint REFERENCES versions ON UPDATE CASCADE,
    mod_dependency_id bigint REFERENCES mods ON UPDATE CASCADE
);