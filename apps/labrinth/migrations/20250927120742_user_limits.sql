ALTER TABLE users
ADD COLUMN max_projects BIGINT,
ADD COLUMN max_organizations BIGINT,
ADD COLUMN max_collections BIGINT;
