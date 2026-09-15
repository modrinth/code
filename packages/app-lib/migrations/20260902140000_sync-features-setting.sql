ALTER TABLE settings
ADD COLUMN sync_features_across_devices INTEGER NOT NULL DEFAULT TRUE;

UPDATE settings
SET sync_features_across_devices = sync_behavior_across_devices;
