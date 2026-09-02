UPDATE sync_feature_settings
SET globally_enabled = 1
WHERE feature IN (
	'game_options',
	'multiplayer_servers',
	'command_history',
	'creative_hotbars'
)
	AND NOT EXISTS (SELECT 1 FROM instances);
