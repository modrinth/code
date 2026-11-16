/*
DO $$
BEGIN
	IF EXISTS (
		SELECT 1
		FROM version_fields vf
		JOIN loader_fields lf ON lf.id = vf.field_id
		WHERE lf.field NOT IN ('game_versions', 'environment', 'mrpack_loaders')
	) THEN
		RAISE EXCEPTION 'cannot migrate unsupported loader fields';
	END IF;

	IF EXISTS (
		SELECT 1
		FROM version_fields vf
		JOIN loader_fields lf ON lf.id = vf.field_id
		LEFT JOIN loader_field_enum_values ev ON ev.id = vf.enum_value
		WHERE vf.int_value IS NOT NULL
			OR vf.string_value IS NOT NULL
			OR ev.id IS NULL
			OR ev.enum_id IS DISTINCT FROM lf.enum_type
			OR (lf.field = 'game_versions' AND lf.field_type <> 'array_enum')
			OR (lf.field = 'environment' AND (
				lf.field_type <> 'enum'
				OR ev.value NOT IN (
					'client_and_server', 'client_only', 'client_only_server_optional',
					'singleplayer_only', 'server_only', 'server_only_client_optional',
					'dedicated_server_only', 'client_or_server',
					'client_or_server_prefers_both', 'unknown'
				)
			))
			OR (lf.field = 'mrpack_loaders' AND (
				lf.field_type <> 'array_enum'
				OR ev.value NOT IN (
					'forge', 'fabric', 'quilt', 'modloader', 'rift', 'liteloader',
					'minecraft', 'bukkit', 'spigot', 'paper', 'purpur', 'bungeecord',
					'waterfall', 'velocity', 'sponge', 'optifine', 'iris', 'canvas',
					'vanilla', 'datapack', 'folia', 'neoforge', 'babric',
					'bta-babric', 'java-agent', 'legacy-fabric', 'nilloader', 'ornithe'
				)
			))
	) THEN
		RAISE EXCEPTION 'cannot migrate invalid Minecraft loader field values';
	END IF;

	IF EXISTS (
		SELECT vf.version_id
		FROM version_fields vf
		JOIN loader_fields lf ON lf.id = vf.field_id
		WHERE lf.field = 'environment'
		GROUP BY vf.version_id
		HAVING COUNT(*) > 1
	) THEN
		RAISE EXCEPTION 'cannot migrate multiple environments for a version';
	END IF;

	IF EXISTS (
		SELECT 1
		FROM versions v
		WHERE v.components->'minecraft_java_server' IS NOT NULL
			AND v.components->'minecraft_java_server' <> 'null'::jsonb
			AND EXISTS (SELECT 1 FROM version_fields vf WHERE vf.version_id = v.id)
	) THEN
		RAISE EXCEPTION 'cannot migrate loader fields on a Minecraft server version';
	END IF;

	IF EXISTS (
		SELECT 1
		FROM versions v
		WHERE v.components->'minecraft_mod' IS NOT NULL
			AND v.components->'minecraft_mod' <> 'null'::jsonb
			AND EXISTS (SELECT 1 FROM version_fields vf WHERE vf.version_id = v.id)
	) THEN
		RAISE EXCEPTION 'cannot overwrite an existing Minecraft mod version component';
	END IF;
END;
$$;

CREATE TABLE minecraft_mod_components_migration (
	version_id BIGINT PRIMARY KEY,
	component JSONB NOT NULL
);

INSERT INTO minecraft_mod_components_migration (version_id, component)
WITH metadata AS (
	SELECT
		vf.version_id,
		COALESCE(
			jsonb_agg(DISTINCT ev.value ORDER BY ev.value)
				FILTER (WHERE lf.field = 'game_versions'),
			'[]'::jsonb
		) AS game_versions,
		MAX(ev.value) FILTER (WHERE lf.field = 'environment') AS environment,
		COALESCE(
			jsonb_agg(DISTINCT ev.value ORDER BY ev.value)
				FILTER (WHERE lf.field = 'mrpack_loaders'),
			'[]'::jsonb
		) AS mrpack_loaders
	FROM version_fields vf
	JOIN loader_fields lf ON lf.id = vf.field_id
	JOIN loader_field_enum_values ev ON ev.id = vf.enum_value
	GROUP BY vf.version_id
)
SELECT
	target.id,
	jsonb_build_object(
		'game_versions', COALESCE(metadata.game_versions, '[]'::jsonb),
		'environment', metadata.environment,
		'mrpack_loaders', COALESCE(metadata.mrpack_loaders, '[]'::jsonb)
	)
FROM versions target
LEFT JOIN metadata ON metadata.version_id = target.id
WHERE (target.components->'minecraft_mod' IS NULL OR target.components->'minecraft_mod' = 'null'::jsonb)
	AND (
		metadata.version_id IS NOT NULL
		OR EXISTS (SELECT 1 FROM loaders_versions lv WHERE lv.version_id = target.id)
	)
	AND (target.components->'minecraft_java_server' IS NULL OR target.components->'minecraft_java_server' = 'null'::jsonb);
*/
