UPDATE sync_feature_settings
SET globally_enabled = 0, new_instance_default = 1
WHERE feature IN (
	'command_history',
	'multiplayer_servers',
	'creative_hotbars'
);

UPDATE instance_sync_preferences
SET enabled = 1
WHERE feature IN (
	'command_history',
	'multiplayer_servers',
	'creative_hotbars'
);
