-- Add migration script here
ALTER TABLE states
ALTER COLUMN url SET NOT NULL;