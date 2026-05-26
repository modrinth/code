DELETE FROM custom_minecraft_skins
WHERE rowid NOT IN (
    SELECT MAX(rowid)
    FROM custom_minecraft_skins
    GROUP BY minecraft_user_uuid, texture_key, variant
);

DROP INDEX IF EXISTS custom_minecraft_skins_unique_without_cape;
DROP INDEX IF EXISTS custom_minecraft_skins_unique_with_cape;

CREATE UNIQUE INDEX IF NOT EXISTS custom_minecraft_skins_one_per_texture_variant
    ON custom_minecraft_skins (minecraft_user_uuid, texture_key, variant);
