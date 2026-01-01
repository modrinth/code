-- Add migration script here
ALTER TABLE settings ADD COLUMN locale TEXT NOT NULL DEFAULT 'en-US';
