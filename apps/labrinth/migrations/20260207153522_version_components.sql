ALTER TABLE versions
ADD COLUMN components JSONB NOT NULL DEFAULT '{}';

-- extra metadata for the `minecraft_java_server` version component
CREATE TABLE minecraft_java_server_versions (
    id          bigint PRIMARY KEY REFERENCES versions(id),
    modpack_id  bigint REFERENCES versions(id)
);
