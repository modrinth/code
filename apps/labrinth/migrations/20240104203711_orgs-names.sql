-- Add migration script here
ALTER TABLE organizations RENAME COLUMN name TO slug;

ALTER TABLE organizations ADD COLUMN name text NULL;
UPDATE organizations SET name = slug;
ALTER TABLE organizations ALTER COLUMN name SET NOT NULL;

