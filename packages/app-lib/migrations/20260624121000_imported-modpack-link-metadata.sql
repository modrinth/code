ALTER TABLE instance_links ADD COLUMN imported_name TEXT NULL;
ALTER TABLE instance_links ADD COLUMN imported_version_number TEXT NULL;
ALTER TABLE instance_links ADD COLUMN imported_filename TEXT NULL;

UPDATE instance_links
SET
	link_kind = 'imported_modpack',
	imported_name = (
		SELECT instances.name
		FROM instances
		WHERE instances.id = instance_links.instance_id
	)
WHERE
	link_kind = 'unmanaged'
	AND EXISTS (
		SELECT 1
		FROM instance_content_entries
		WHERE
			instance_content_entries.instance_id = instance_links.instance_id
			AND instance_content_entries.source_kind = 'imported_modpack'
	);
