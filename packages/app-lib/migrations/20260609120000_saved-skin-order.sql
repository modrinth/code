ALTER TABLE custom_minecraft_skins
ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0;

UPDATE custom_minecraft_skins
SET display_order = rowid;
