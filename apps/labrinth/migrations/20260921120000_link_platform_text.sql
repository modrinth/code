ALTER TABLE mods_links
	ADD COLUMN platform TEXT;

DO $$
BEGIN
	IF EXISTS (
		SELECT 1
		FROM link_platforms
		WHERE name NOT IN (
			'patreon',
			'bmac',
			'paypal',
			'github',
			'ko-fi',
			'other',
			'issues',
			'wiki',
			'discord',
			'source',
			'site',
			'store'
		)
	) THEN
		RAISE EXCEPTION 'invalid link platform found';
	END IF;

	UPDATE mods_links ml
	SET platform = CASE lp.name
		WHEN 'patreon' THEN 'patreon'
		WHEN 'bmac' THEN 'bmac'
		WHEN 'paypal' THEN 'paypal'
		WHEN 'github' THEN 'github'
		WHEN 'ko-fi' THEN 'kofi'
		WHEN 'other' THEN 'other'
		WHEN 'issues' THEN 'issues'
		WHEN 'wiki' THEN 'wiki'
		WHEN 'discord' THEN 'discord'
		WHEN 'source' THEN 'source'
		WHEN 'site' THEN 'site'
		WHEN 'store' THEN 'store'
	END
	FROM link_platforms lp
	WHERE lp.id = ml.joining_platform_id;
END
$$;

ALTER TABLE mods_links
	ALTER COLUMN platform SET NOT NULL,
	DROP COLUMN joining_platform_id;

DROP TABLE link_platforms;
