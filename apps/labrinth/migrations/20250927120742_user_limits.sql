CREATE TABLE user_limits (
	-- if NULL, this is a global default
	user_id  	  BIGINT  REFERENCES users(id),
	projects 	  INTEGER NOT NULL,
	organizations INTEGER NOT NULL,
	collections   INTEGER NOT NULL
);
INSERT INTO user_limits (user_id, projects, organizations, collections)
VALUES (NULL, 256, 16, 32);
