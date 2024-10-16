-- Add migration script here
CREATE TABLE mods_gallery (
    id serial PRIMARY KEY,
    mod_id bigint REFERENCES mods ON UPDATE CASCADE NOT NULL,
    image_url varchar(2048) NOT NULL
);