CREATE TABLE statuses (
    id serial PRIMARY KEY UNIQUE NOT NULL,
    status varchar(64) UNIQUE NOT NULL
);

ALTER TABLE mods
ADD COLUMN status integer REFERENCES statuses NOT NULL;
ALTER TABLE mods
ADD COLUMN updated timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP;

INSERT INTO statuses (status) VALUES ('approved');
INSERT INTO statuses (status) VALUES ('rejected');
INSERT INTO statuses (status) VALUES ('draft');
INSERT INTO statuses (status) VALUES ('unlisted');
INSERT INTO statuses (status) VALUES ('processing');