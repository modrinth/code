ALTER TABLE users_compliance ADD COLUMN requires_manual_review BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users_compliance ALTER COLUMN form_type DROP NOT NULL;
