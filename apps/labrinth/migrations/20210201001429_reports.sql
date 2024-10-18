CREATE TABLE report_types (
    id serial PRIMARY KEY,
    name varchar(64) UNIQUE NOT NULL
);

INSERT INTO report_types (name) VALUES ('spam');
INSERT INTO report_types (name) VALUES ('copyright');
INSERT INTO report_types (name) VALUES ('inappropriate');
INSERT INTO report_types (name) VALUES ('malicious');
INSERT INTO report_types (name) VALUES ('name-squatting');

CREATE TABLE reports (
    id bigint PRIMARY KEY,
    report_type_id int REFERENCES report_types ON UPDATE CASCADE NOT NULL,
    mod_id bigint REFERENCES mods ON UPDATE CASCADE,
    version_id bigint REFERENCES versions ON UPDATE CASCADE,
    user_id bigint REFERENCES users ON UPDATE CASCADE,
    body varchar(65536) NOT NULL,
    reporter bigint REFERENCES users ON UPDATE CASCADE NOT NULL,
    created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL
);

ALTER TABLE game_versions
    ADD COLUMN major boolean NOT NULL DEFAULT FALSE;