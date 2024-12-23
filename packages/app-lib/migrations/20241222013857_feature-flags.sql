ALTER TABLE settings ADD COLUMN toggle_sidebar INTEGER NOT NULL DEFAULT FALSE;
ALTER TABLE settings ADD COLUMN feature_flags JSONB NOT NULL default '{}';