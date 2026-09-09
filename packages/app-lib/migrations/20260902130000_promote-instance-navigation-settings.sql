ALTER TABLE settings
ADD COLUMN show_files_tab_in_instances INTEGER NOT NULL DEFAULT TRUE CHECK (show_files_tab_in_instances IN (0, 1));

ALTER TABLE settings
ADD COLUMN show_worlds_tab_in_instances INTEGER NOT NULL DEFAULT TRUE CHECK (show_worlds_tab_in_instances IN (0, 1));

ALTER TABLE settings
ADD COLUMN show_screenshots_tab_in_instances INTEGER NOT NULL DEFAULT FALSE CHECK (show_screenshots_tab_in_instances IN (0, 1));

ALTER TABLE settings
ADD COLUMN show_skin_selector_in_sidebar INTEGER NOT NULL DEFAULT TRUE CHECK (show_skin_selector_in_sidebar IN (0, 1));

UPDATE settings
SET
	show_files_tab_in_instances = COALESCE(json_extract(feature_flags, '$.show_files_tab_in_instances'), TRUE),
	show_worlds_tab_in_instances = COALESCE(json_extract(feature_flags, '$.show_worlds_tab_in_instances'), TRUE),
	show_screenshots_tab_in_instances = COALESCE(json_extract(feature_flags, '$.show_screenshots_tab_in_instances'), FALSE),
	show_skin_selector_in_sidebar = COALESCE(json_extract(feature_flags, '$.show_skin_selector_in_sidebar'), TRUE),
	feature_flags = json_remove(
		feature_flags,
		'$.show_files_tab_in_instances',
		'$.show_worlds_tab_in_instances',
		'$.show_screenshots_tab_in_instances',
		'$.show_skin_selector_in_sidebar'
	);
