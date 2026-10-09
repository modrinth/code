INSERT INTO loader_field_enum_values (enum_id, value)
SELECT id, 'ornithe' FROM loader_field_enums WHERE enum_name = 'mrpack_loaders'
ON CONFLICT (enum_id, value) DO NOTHING;
