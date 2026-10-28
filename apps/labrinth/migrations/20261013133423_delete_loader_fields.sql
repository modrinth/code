/*
UPDATE versions v
SET components = jsonb_set(
	v.components,
	'{minecraft_mod}',
	staged.component
)
FROM minecraft_mod_components_migration staged
WHERE v.id = staged.version_id;

DROP TABLE minecraft_mod_components_migration;
DROP TABLE version_fields;
DROP TABLE loader_fields_loaders;
DROP TABLE loader_fields;
DROP TABLE loader_field_enum_values;
DROP TABLE loader_field_enums;
DROP TABLE loaders_project_types_games;
DROP TABLE games;
*/
