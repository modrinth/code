ALTER TABLE mods
ADD COLUMN components JSONB NOT NULL DEFAULT '{}';

CREATE TABLE minecraft_server_projects (
	id bigint PRIMARY KEY NOT NULL
		REFERENCES mods(id)
		ON DELETE CASCADE,
	max_players int NOT NULL
);

CREATE TABLE minecraft_java_server_projects (
	id bigint PRIMARY KEY NOT NULL
		REFERENCES mods(id)
		ON DELETE CASCADE,
	address varchar(255) NOT NULL
);

CREATE TABLE minecraft_bedrock_server_projects (
	id bigint PRIMARY KEY NOT NULL
		REFERENCES mods(id)
		ON DELETE CASCADE,
	address varchar(255) NOT NULL
);
