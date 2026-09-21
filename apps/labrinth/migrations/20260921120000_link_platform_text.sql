ALTER TABLE mods_links
	ADD COLUMN platform TEXT;

/*
UPDATE mods_links ml
SET platform = lp.name
FROM link_platforms lp
WHERE lp.id = ml.joining_platform_id;
*/
