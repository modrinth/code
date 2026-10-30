ALTER TABLE settings
ADD COLUMN refocus_on_game_close INTEGER NOT NULL DEFAULT FALSE CHECK (refocus_on_game_close IN (0, 1));

ALTER TABLE settings
ADD COLUMN compact_instance_cards INTEGER NOT NULL DEFAULT FALSE CHECK (compact_instance_cards IN (0, 1));

ALTER TABLE settings
ADD COLUMN show_play_time INTEGER NOT NULL DEFAULT TRUE CHECK (show_play_time IN (0, 1));

ALTER TABLE settings
ADD COLUMN warn_on_unknown_modpacks INTEGER NOT NULL DEFAULT TRUE CHECK (warn_on_unknown_modpacks IN (0, 1));

ALTER TABLE settings
ADD COLUMN skip_non_essential_warnings INTEGER NOT NULL DEFAULT FALSE CHECK (skip_non_essential_warnings IN (0, 1));

ALTER TABLE settings
ADD COLUMN show_jump_in INTEGER NOT NULL DEFAULT TRUE CHECK (show_jump_in IN (0, 1));

ALTER TABLE settings
ADD COLUMN always_show_copy_details INTEGER NOT NULL DEFAULT FALSE CHECK (always_show_copy_details IN (0, 1));

ALTER TABLE settings
ADD COLUMN hide_installed_modpacks INTEGER NOT NULL DEFAULT FALSE CHECK (hide_installed_modpacks IN (0, 1));

ALTER TABLE settings
ADD COLUMN advanced_filters_collapsed INTEGER NOT NULL DEFAULT TRUE CHECK (advanced_filters_collapsed IN (0, 1));

ALTER TABLE settings
ADD COLUMN dismissed_photosensitivity_filter_warning INTEGER NOT NULL DEFAULT FALSE CHECK (dismissed_photosensitivity_filter_warning IN (0, 1));

ALTER TABLE settings
ADD COLUMN friends_active_collapsed INTEGER NOT NULL DEFAULT FALSE CHECK (friends_active_collapsed IN (0, 1));

ALTER TABLE settings
ADD COLUMN friends_online_collapsed INTEGER NOT NULL DEFAULT FALSE CHECK (friends_online_collapsed IN (0, 1));

ALTER TABLE settings
ADD COLUMN friends_offline_collapsed INTEGER NOT NULL DEFAULT TRUE CHECK (friends_offline_collapsed IN (0, 1));

ALTER TABLE settings
ADD COLUMN friends_pending_collapsed INTEGER NOT NULL DEFAULT TRUE CHECK (friends_pending_collapsed IN (0, 1));

UPDATE settings
SET
	refocus_on_game_close = COALESCE(json_extract(feature_flags, '$.refocus_on_game_close'), FALSE),
	compact_instance_cards = COALESCE(json_extract(feature_flags, '$.compact_instance_cards'), FALSE),
	show_play_time = COALESCE(json_extract(feature_flags, '$.show_instance_play_time'), TRUE),
	warn_on_unknown_modpacks = NOT COALESCE(json_extract(feature_flags, '$.skip_unknown_pack_warning'), FALSE),
	skip_non_essential_warnings = COALESCE(json_extract(feature_flags, '$.skip_non_essential_warnings'), FALSE),
	show_jump_in = COALESCE(json_extract(feature_flags, '$.worlds_in_home'), TRUE),
	always_show_copy_details = COALESCE(json_extract(feature_flags, '$.always_show_copy_details'), FALSE),
	hide_installed_modpacks = COALESCE(json_extract(feature_flags, '$.hide_installed_modpacks'), FALSE),
	advanced_filters_collapsed = COALESCE(json_extract(feature_flags, '$.advanced_filters_collapsed'), TRUE),
	dismissed_photosensitivity_filter_warning = COALESCE(json_extract(feature_flags, '$.dismissed_photosensitivity_filter_warning'), FALSE),
	friends_active_collapsed = COALESCE(json_extract(feature_flags, '$.friends_active_collapsed'), FALSE),
	friends_online_collapsed = COALESCE(json_extract(feature_flags, '$.friends_online_collapsed'), FALSE),
	friends_offline_collapsed = COALESCE(json_extract(feature_flags, '$.friends_offline_collapsed'), TRUE),
	friends_pending_collapsed = COALESCE(json_extract(feature_flags, '$.friends_pending_collapsed'), TRUE),
	feature_flags = json_remove(
		feature_flags,
		'$.refocus_on_game_close',
		'$.compact_instance_cards',
		'$.show_instance_play_time',
		'$.skip_unknown_pack_warning',
		'$.skip_non_essential_warnings',
		'$.worlds_in_home',
		'$.always_show_copy_details',
		'$.hide_installed_modpacks',
		'$.advanced_filters_collapsed',
		'$.dismissed_photosensitivity_filter_warning',
		'$.friends_active_collapsed',
		'$.friends_online_collapsed',
		'$.friends_offline_collapsed',
		'$.friends_pending_collapsed'
	);
