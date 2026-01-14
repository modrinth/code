CREATE TABLE minecraft_server_projects (
	id              bigint PRIMARY KEY NOT NULL REFERENCES mods(id),
	java_address    varchar(255) NOT NULL,
	bedrock_address varchar(255) NOT NULL,
	max_players     int
);
