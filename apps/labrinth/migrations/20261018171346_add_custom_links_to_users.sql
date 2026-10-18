-- Add migration script here
ALTER TABLE users
  ADD COLUMN custom_links JSONB NOT NULL DEFAULT '[]'::JSONB;
