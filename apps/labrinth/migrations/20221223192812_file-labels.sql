-- Add migration script here
ALTER TABLE files ADD COLUMN file_type varchar(128) NULL;