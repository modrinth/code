-- Keep only one default cape per Minecraft account.
DELETE FROM default_minecraft_capes
WHERE rowid NOT IN (
    SELECT MAX(rowid)
    FROM default_minecraft_capes
    GROUP BY minecraft_user_uuid
);

CREATE UNIQUE INDEX default_minecraft_capes_one_per_user
    ON default_minecraft_capes (minecraft_user_uuid);

-- Keep only one saved skin per Minecraft account, texture, and model.
-- cape_id is a setting on that saved skin, not part of the skin identity.
DELETE FROM custom_minecraft_skins
WHERE rowid NOT IN (
    SELECT MAX(rowid)
    FROM custom_minecraft_skins
    GROUP BY minecraft_user_uuid, texture_key, variant
);

CREATE UNIQUE INDEX custom_minecraft_skins_one_per_texture_variant
    ON custom_minecraft_skins (minecraft_user_uuid, texture_key, variant);
