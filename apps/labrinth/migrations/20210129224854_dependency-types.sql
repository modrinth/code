-- Add migration script here
ALTER TABLE dependencies
    ADD COLUMN dependency_type varchar(255) NOT NULL DEFAULT 'required';