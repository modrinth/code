ALTER TABLE instance_links
	ADD COLUMN shared_instance_restored_from_remote INTEGER NOT NULL DEFAULT 0;
