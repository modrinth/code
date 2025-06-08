ALTER TABLE shared_instance_versions
ADD COLUMN created timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP;
