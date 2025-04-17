CREATE TABLE default_minecraft_capes (
    minecraft_user_uuid TEXT NOT NULL,
    id TEXT NOT NULL,

    PRIMARY KEY (minecraft_user_uuid, id),
    FOREIGN KEY (minecraft_user_uuid) REFERENCES minecraft_users(uuid)
        ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE custom_minecraft_skins (
    minecraft_user_uuid TEXT NOT NULL,
    texture_key TEXT NOT NULL,
    variant TEXT NOT NULL CHECK (variant IN ('CLASSIC', 'SLIM', 'UNKNOWN')),
    cape_id TEXT,

    PRIMARY KEY (minecraft_user_uuid, texture_key, variant, cape_id),
    FOREIGN KEY (minecraft_user_uuid) REFERENCES minecraft_users(uuid)
        ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (texture_key) REFERENCES custom_minecraft_skin_textures(texture_key)
        ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE custom_minecraft_skin_textures (
    texture_key TEXT NOT NULL,
    texture PNG BLOB NOT NULL,

    PRIMARY KEY (texture_key)
);

CREATE TRIGGER custom_minecraft_skin_texture_delete_cleanup
    AFTER DELETE ON custom_minecraft_skins FOR EACH ROW
    BEGIN
        DELETE FROM custom_minecraft_skin_textures WHERE texture_key NOT IN (
            SELECT texture_key FROM custom_minecraft_skins
        );
    END;
