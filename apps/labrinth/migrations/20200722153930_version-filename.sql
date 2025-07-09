-- Add migration script here
ALTER TABLE files
ADD filename varchar(2048) NOT NULL;
