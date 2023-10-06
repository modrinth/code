-- Dummy test data for use in tests.
-- IDs are listed as integers, followed by their equivalent base 62 representation.

-- Inserts 5 dummy users for testing, with slight differences
-- 'Friend' and 'enemy' function like 'user', but we can use them to simulate 'other' users that may or may not be able to access certain things
-- IDs 1-5, 1-5
INSERT INTO users (id, username, name, email, role) VALUES (1, 'admin', 'Administrator Test', 'admin@modrinth.com', 'admin');
INSERT INTO users (id, username, name, email, role) VALUES (2, 'moderator', 'Moderator Test', 'moderator@modrinth.com', 'moderator');
INSERT INTO users (id, username, name, email, role) VALUES (3, 'user', 'User Test', 'user@modrinth.com', 'developer');
INSERT INTO users (id, username, name, email, role) VALUES (4, 'friend', 'Friend Test', 'friend@modrinth.com', 'developer');
INSERT INTO users (id, username, name, email, role) VALUES (5, 'enemy', 'Enemy Test', 'enemy@modrinth.com', 'developer');

-- Full PATs for each user, with different scopes
-- These are not legal PATs, as they contain all scopes- they mimic permissions of a logged in user
-- IDs: 50-54, o p q r s
INSERT INTO pats (id, user_id, name, access_token, scopes, expires) VALUES (50, 1, 'admin-pat', 'mrp_patadmin', B'11111111111111111111111111111111111'::BIGINT, '2030-08-18 15:48:58.435729+00');
INSERT INTO pats (id, user_id, name, access_token, scopes, expires) VALUES (51, 2, 'moderator-pat', 'mrp_patmoderator', B'11111111111111111111111111111111111'::BIGINT, '2030-08-18 15:48:58.435729+00');
INSERT INTO pats (id, user_id, name, access_token, scopes, expires) VALUES (52, 3, 'user-pat', 'mrp_patuser', B'11111111111111111111111111111111111'::BIGINT, '2030-08-18 15:48:58.435729+00');
INSERT INTO pats (id, user_id, name, access_token, scopes, expires) VALUES (53, 4, 'friend-pat', 'mrp_patfriend', B'11111111111111111111111111111111111'::BIGINT, '2030-08-18 15:48:58.435729+00');
INSERT INTO pats (id, user_id, name, access_token, scopes, expires) VALUES (54, 5, 'enemy-pat', 'mrp_patenemy', B'11111111111111111111111111111111111'::BIGINT, '2030-08-18 15:48:58.435729+00');

-- -- Sample game versions, loaders, categories
INSERT INTO game_versions (id, version, type, created)
VALUES (20000, '1.20.1', 'release', timezone('utc', now()));

INSERT INTO loaders (id, loader) VALUES (1, 'fabric');
INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id) VALUES (1,1);
INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id) VALUES (1,2); 

INSERT INTO categories (id, category, project_type) VALUES (1, 'combat', 1);
INSERT INTO categories (id, category, project_type) VALUES (2, 'decoration', 1);
INSERT INTO categories (id, category, project_type) VALUES (3, 'economy', 1);

INSERT INTO categories (id, category, project_type) VALUES (4, 'combat', 2);
INSERT INTO categories (id, category, project_type) VALUES (5, 'decoration', 2);
INSERT INTO categories (id, category, project_type) VALUES (6, 'economy', 2);