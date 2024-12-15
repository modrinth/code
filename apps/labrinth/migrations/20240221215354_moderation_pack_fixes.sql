ALTER TABLE moderation_external_files ALTER COLUMN sha1 SET NOT NULL;
ALTER TABLE moderation_external_licenses ALTER COLUMN title DROP NOT NULL;
