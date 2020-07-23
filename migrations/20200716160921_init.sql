CREATE TABLE users (
    -- TODO
    id bigint PRIMARY KEY
);

CREATE TABLE game_versions (
    id serial PRIMARY KEY,
    version varchar(255) NOT NULL
);

CREATE TABLE loaders (
    id serial PRIMARY KEY,
    loader varchar(255) NOT NULL
);

CREATE TABLE teams (
    id bigint PRIMARY KEY
);

CREATE TABLE release_channel (
    id serial PRIMARY KEY,
    channel varchar(255)
);

CREATE TABLE mods (
    id bigint PRIMARY KEY,
    team_id bigint REFERENCES teams NOT NULL,
    title varchar(255) NOT NULL,
    description varchar(2048) NOT NULL,
    body_url varchar(2048) NOT NULL,
    published timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    downloads integer NOT NULL DEFAULT 0,
    
    icon_url varchar(2048) NULL,
    issues_url varchar(2048) NULL,
    source_url varchar(2048) NULL,
    wiki_url varchar(2048) NULL
);


CREATE TABLE versions (
    id bigint PRIMARY KEY,
    mod_id bigint REFERENCES mods,
    name varchar(255) NOT NULL,
    version_number varchar(255) NOT NULL,
    changelog_url varchar(255) NULL,
    date_published timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    downloads integer NOT NULL DEFAULT 0,

    release_channel int REFERENCES release_channel ON UPDATE CASCADE NOT NULL
);

CREATE TABLE loaders_versions (
    loader_id int REFERENCES loaders ON UPDATE CASCADE NOT NULL,
    version_id bigint REFERENCES versions ON UPDATE CASCADE NOT NULL,
    PRIMARY KEY (loader_id, version_id)
);

CREATE TABLE game_versions_versions (
    game_version_id integer REFERENCES game_versions ON UPDATE CASCADE NOT NULL,
    joining_version_id bigint REFERENCES versions ON UPDATE CASCADE NOT NULL,
    PRIMARY KEY (game_version_id, joining_version_id)
);

CREATE TABLE files (
    id bigint PRIMARY KEY,
    version_id bigint REFERENCES versions NOT NULL,
    url varchar(2048) NOT NULL
);

CREATE TABLE hashes (
    file_id bigint REFERENCES files NOT NULL,
    algorithm varchar(255) NOT NULL,
    hash bytea NOT NULL,
    PRIMARY KEY (file_id, algorithm)
);

CREATE TABLE dependencies (
    id serial PRIMARY KEY,
    dependent_id bigint REFERENCES versions ON UPDATE CASCADE NOT NULL,
    dependency_id bigint REFERENCES versions ON UPDATE CASCADE NOT NULL,
    CONSTRAINT valid_dependency CHECK (dependent_id <> dependency_id) -- No dependency on yourself
);

CREATE TABLE team_members (
    id bigint PRIMARY KEY,
    team_id bigint REFERENCES teams NOT NULL,
    user_id bigint REFERENCES users NOT NULL,
    member_name varchar(255) NOT NULL,
    role varchar(255) NOT NULL
);

CREATE TABLE categories (
    id serial PRIMARY KEY,
    category varchar(255) UNIQUE
);

CREATE TABLE mods_categories (
    joining_mod_id bigint REFERENCES mods ON UPDATE CASCADE NOT NULL,
    joining_category_id int REFERENCES categories ON UPDATE CASCADE NOT NULL,
    PRIMARY KEY (joining_mod_id, joining_category_id)
);
