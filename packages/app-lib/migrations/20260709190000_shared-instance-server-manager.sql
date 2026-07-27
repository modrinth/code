ALTER TABLE instance_links
	ADD COLUMN shared_instance_server_manager_name TEXT NULL;

ALTER TABLE instance_links
	ADD COLUMN shared_instance_server_manager_icon_url TEXT NULL;
