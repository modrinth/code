ALTER TABLE moderation_external_licenses
    ADD COLUMN inserted_at timestamptz,
    ADD COLUMN inserted_by bigint REFERENCES users(id),
    ADD COLUMN updated_at timestamptz,
    ADD COLUMN updated_by bigint REFERENCES users(id);

ALTER TABLE moderation_external_files
	ADD COLUMN filename text,
    ADD COLUMN inserted_at timestamptz,
    ADD COLUMN inserted_by bigint REFERENCES users(id),
    ADD COLUMN updated_at timestamptz,
    ADD COLUMN updated_by bigint REFERENCES users(id);
