-- Add migration script here
ALTER TABLE users
ADD COLUMN role varchar(50) NOT NULL default 'developer'