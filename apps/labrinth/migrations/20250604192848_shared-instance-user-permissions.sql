ALTER TABLE shared_instance_users
ADD COLUMN permissions BIGINT NOT NULL DEFAULT 0;
