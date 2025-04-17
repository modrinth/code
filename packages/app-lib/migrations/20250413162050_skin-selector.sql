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
        ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE custom_minecraft_skin_textures (
    texture_key TEXT NOT NULL,
    texture PNG BLOB NOT NULL,

    PRIMARY KEY (texture_key)
);

-- Use triggers to emulate partial cascading foreign key constraints on the custom_minecraft_skin_textures table

CREATE TRIGGER custom_minecraft_skin_texture_insertion_validation
    BEFORE INSERT ON custom_minecraft_skin_textures FOR EACH ROW
    BEGIN
        SELECT CASE WHEN NOT EXISTS (
            SELECT 1 FROM custom_minecraft_skins WHERE texture_key = NEW.texture_key
        ) THEN RAISE(ABORT, 'Missing custom skin for the specified skin texture key') END;
    END;

CREATE TRIGGER custom_minecraft_skin_texture_delete_cleanup
    AFTER DELETE ON custom_minecraft_skins FOR EACH ROW
    BEGIN
        DELETE FROM custom_minecraft_skin_textures WHERE texture_key NOT IN (
            SELECT texture_key FROM custom_minecraft_skins
        );
    END;

CREATE TRIGGER custom_minecraft_skin_texture_update_cleanup
    AFTER UPDATE OF texture_key ON custom_minecraft_skins FOR EACH ROW
    BEGIN
        UPDATE custom_minecraft_skin_textures SET texture_key = NEW.texture_key WHERE texture_key = OLD.texture_key;
    END;
