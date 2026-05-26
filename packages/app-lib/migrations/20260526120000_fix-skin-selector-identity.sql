DELETE FROM default_minecraft_capes
WHERE rowid NOT IN (
    SELECT MAX(rowid)
    FROM default_minecraft_capes
    GROUP BY minecraft_user_uuid
);

CREATE UNIQUE INDEX default_minecraft_capes_one_per_user
    ON default_minecraft_capes (minecraft_user_uuid);

DELETE FROM custom_minecraft_skins
WHERE rowid NOT IN (
    SELECT MIN(rowid)
    FROM custom_minecraft_skins
    GROUP BY minecraft_user_uuid, texture_key, variant, cape_id
);

CREATE UNIQUE INDEX custom_minecraft_skins_unique_without_cape
    ON custom_minecraft_skins (minecraft_user_uuid, texture_key, variant)
    WHERE cape_id IS NULL;

CREATE UNIQUE INDEX custom_minecraft_skins_unique_with_cape
    ON custom_minecraft_skins (minecraft_user_uuid, texture_key, variant, cape_id)
    WHERE cape_id IS NOT NULL;
