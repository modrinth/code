CREATE TABLE game_option_locale_origins (
	scope TEXT NOT NULL,
	option_id TEXT NOT NULL,
	source_instance_id TEXT,
	source_game_version TEXT,
	backfilled INTEGER NOT NULL DEFAULT 0,
	observation_json TEXT,
	origin_json TEXT,
	PRIMARY KEY (scope, option_id)
);

INSERT INTO game_option_locale_origins
	(scope, option_id, source_instance_id, source_game_version, backfilled)
SELECT '', option_id, source_instance_id, source_game_version, 1
FROM synced_game_option_values;

CREATE TRIGGER record_game_option_locale_origin
AFTER INSERT ON synced_game_option_values
BEGIN
	INSERT INTO game_option_locale_origins
		(scope, option_id, source_instance_id, source_game_version)
	VALUES ('', NEW.option_id, NEW.source_instance_id, NEW.source_game_version)
	ON CONFLICT(scope, option_id) DO NOTHING;
END;
