INSERT INTO sync_feature_settings
	(feature, globally_enabled, new_instance_default)
VALUES
	('resource_packs', 0, 1),
	('data_packs', 0, 1);

INSERT INTO instance_sync_preferences (instance_id, feature, enabled)
SELECT instances.id, features.feature, 1
FROM instances
CROSS JOIN sync_feature_settings features
WHERE features.feature IN ('resource_packs', 'data_packs');
