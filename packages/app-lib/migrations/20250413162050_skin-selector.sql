CREATE TABLE default_minecraft_capes (
    minecraft_user_uuid TEXT NOT NULL,
    id TEXT NOT NULL,

    PRIMARY KEY (minecraft_user_uuid, id)
);

-- Emulate a ON UPDATE CASCADE foreign key constraint for the user UUID on the default_minecraft_capes table,
-- but allowing deletion of the user UUID in the minecraft_users table. This allows the application to temporarily
-- keep skin state around for logged-out users, allowing them to retain their skins under the right conditions
CREATE TRIGGER default_minecraft_capes_user_uuid_insert_check
    BEFORE INSERT ON default_minecraft_capes FOR EACH ROW
    BEGIN
        SELECT CASE WHEN NOT EXISTS (
            SELECT 1 FROM minecraft_users WHERE uuid = NEW.minecraft_user_uuid
        ) THEN RAISE(ABORT, 'Cannot add a default cape for an unknown Minecraft user UUID') END;
    END;

CREATE TRIGGER default_minecraft_capes_user_uuid_update_check
    BEFORE UPDATE ON default_minecraft_capes FOR EACH ROW
    BEGIN
        SELECT CASE WHEN NOT EXISTS (
            SELECT 1 FROM minecraft_users WHERE uuid = NEW.minecraft_user_uuid
        ) THEN RAISE(ABORT, 'Cannot change a default cape to refer to an unknown Minecraft user UUID') END;
    END;

CREATE TRIGGER default_minecraft_capes_user_uuid_update_cascade
    AFTER UPDATE OF uuid ON minecraft_users FOR EACH ROW
    BEGIN
        UPDATE default_minecraft_capes SET minecraft_user_uuid = NEW.uuid WHERE minecraft_user_uuid = OLD.uuid;
    END;

CREATE TABLE custom_minecraft_skins (
    minecraft_user_uuid TEXT NOT NULL,
    texture_key TEXT NOT NULL,
    variant TEXT NOT NULL CHECK (variant IN ('CLASSIC', 'SLIM', 'UNKNOWN')),
    cape_id TEXT,

    PRIMARY KEY (minecraft_user_uuid, texture_key, variant, cape_id),
    FOREIGN KEY (texture_key) REFERENCES custom_minecraft_skin_textures(texture_key)
        ON DELETE CASCADE ON UPDATE CASCADE
);

-- Similar partial foreign key emulation as above
CREATE TRIGGER custom_minecraft_skins_user_uuid_insert_check
    BEFORE INSERT ON custom_minecraft_skins FOR EACH ROW
    BEGIN
        SELECT CASE WHEN NOT EXISTS (
            SELECT 1 FROM minecraft_users WHERE uuid = NEW.minecraft_user_uuid
        ) THEN RAISE(ABORT, 'Cannot add a custom skin for an unknown Minecraft user UUID') END;
    END;

CREATE TRIGGER custom_minecraft_skins_user_uuid_update_check
    BEFORE UPDATE ON custom_minecraft_skins FOR EACH ROW
    BEGIN
        SELECT CASE WHEN NOT EXISTS (
            SELECT 1 FROM minecraft_users WHERE uuid = NEW.minecraft_user_uuid
        ) THEN RAISE(ABORT, 'Cannot change a custom skin to refer to an unknown Minecraft user UUID') END;
    END;

CREATE TRIGGER custom_minecraft_skins_user_uuid_update_cascade
    AFTER UPDATE OF uuid ON minecraft_users FOR EACH ROW
    BEGIN
        UPDATE custom_minecraft_skins SET minecraft_user_uuid = NEW.uuid WHERE minecraft_user_uuid = OLD.uuid;
    END;

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
