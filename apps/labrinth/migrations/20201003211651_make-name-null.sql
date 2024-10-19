-- Add migration script here
ALTER TABLE users
ALTER COLUMN name DROP NOT NULL;
ALTER TABLE users
ALTER COLUMN name DROP DEFAULT;
