-- Add migration script here
ALTER TABLE categories
ALTER COLUMN category SET NOT NULL;
