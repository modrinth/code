ALTER TABLE custom_minecraft_skins
ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0;

UPDATE custom_minecraft_skins
SET display_order = (
	SELECT COUNT(*)
	FROM custom_minecraft_skins AS previous
	WHERE previous.minecraft_user_uuid = custom_minecraft_skins.minecraft_user_uuid
		AND previous.rowid <= custom_minecraft_skins.rowid
) - 1;

CREATE INDEX custom_minecraft_skins_user_display_order
	ON custom_minecraft_skins (minecraft_user_uuid, display_order);
