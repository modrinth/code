ALTER TABLE settings
ADD COLUMN skipped_update TEXT NULL;
ALTER TABLE settings
ADD COLUMN pending_update_toast_for_version TEXT NULL;
ALTER TABLE settings
ADD COLUMN auto_download_updates INT NULL;
