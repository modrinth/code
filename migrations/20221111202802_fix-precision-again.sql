-- Add migration script here
ALTER TABLE users ALTER balance TYPE numeric(40, 20);