-- Add migration script here

ALTER TABLE sessions ALTER COLUMN expires DROP DEFAULT;
ALTER TABLE sessions ALTER COLUMN refresh_expires DROP DEFAULT;
