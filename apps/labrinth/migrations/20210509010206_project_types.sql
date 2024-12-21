ALTER TABLE users ADD CONSTRAINT username_unique UNIQUE (username);

CREATE TABLE project_types (
    id serial PRIMARY KEY,
    name varchar(64) UNIQUE NOT NULL
);

INSERT INTO project_types (name) VALUES ('mod');
INSERT INTO project_types (name) VALUES ('modpack');

CREATE TABLE loaders_project_types (
    joining_loader_id int REFERENCES loaders ON UPDATE CASCADE NOT NULL,
    joining_project_type_id int REFERENCES project_types ON UPDATE CASCADE NOT NULL,
    PRIMARY KEY (joining_loader_id, joining_project_type_id)
);

ALTER TABLE mods
    ADD COLUMN project_type integer REFERENCES project_types NOT NULL default 1;

ALTER TABLE categories
    ADD COLUMN project_type integer REFERENCES project_types NOT NULL default 1,
    ADD COLUMN icon varchar(20000) NOT NULL default '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>';

ALTER TABLE loaders
    ADD COLUMN icon varchar(20000) NOT NULL default '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>';

ALTER TABLE mods
    ALTER COLUMN project_type DROP DEFAULT;

ALTER TABLE categories
    ALTER COLUMN project_type DROP DEFAULT;
