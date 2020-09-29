-- Add migration script here
ALTER TABLE versions
ADD COLUMN author_id bigint REFERENCES users NOT NULL default 0