DROP TABLE IF EXISTS default_minecraft_capes;

-- Keep only one saved skin per Minecraft account and texture.
-- variant and cape_id are settings on that saved skin, not part of the skin identity.
DELETE FROM custom_minecraft_skins
WHERE rowid NOT IN (
    SELECT MAX(rowid)
    FROM custom_minecraft_skins
    GROUP BY minecraft_user_uuid, texture_key
);

CREATE UNIQUE INDEX custom_minecraft_skins_one_per_texture
    ON custom_minecraft_skins (minecraft_user_uuid, texture_key);
