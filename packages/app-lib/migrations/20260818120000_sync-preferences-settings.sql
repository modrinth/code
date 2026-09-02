ALTER TABLE settings
ADD COLUMN sync_theme_across_devices INTEGER NOT NULL DEFAULT TRUE;

ALTER TABLE settings
ADD COLUMN sync_behavior_across_devices INTEGER NOT NULL DEFAULT TRUE;
