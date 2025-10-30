-- Dummy test data for use in tests.
-- IDs are listed as integers, followed by their equivalent base 62 representation.

-- Inserts 5 dummy users for testing, with slight differences
-- 'Friend' and 'enemy' function like 'user', but we can use them to simulate 'other' users that may or may not be able to access certain things
INSERT INTO users (id, username, email, role)
VALUES ({{user_id::ADMIN}},     'Admin',     'admin@modrinth.com',     'admin'),
       ({{user_id::MODERATOR}}, 'Moderator', 'moderator@modrinth.com', 'moderator'),
       ({{user_id::USER}},      'User',      'user@modrinth.com',      'developer'),
       ({{user_id::FRIEND}},    'Friend',    'friend@modrinth.com',    'developer'),
       ({{user_id::ENEMY}},     'Enemy',     'enemy@modrinth.com',     'developer');

-- Full PATs for each user, with different scopes
-- These are not legal PATs, as they contain all scopes- they mimic permissions of a logged in user
-- IDs: 50-54, o p q r s
INSERT INTO pats (id, user_id, name, access_token, scopes, expires)
VALUES (50, {{user_id::ADMIN}},     'admin-pat',     '{{pat::ADMIN}}',     {{all_scopes}}, '2030-08-18 15:48:58.435729+00'),
       (51, {{user_id::MODERATOR}}, 'moderator-pat', '{{pat::MODERATOR}}', {{all_scopes}}, '2030-08-18 15:48:58.435729+00'),
       (52, {{user_id::USER}},      'user-pat',      '{{pat::USER}}',      {{all_scopes}}, '2030-08-18 15:48:58.435729+00'),
       (53, {{user_id::FRIEND}},    'friend-pat',    '{{pat::FRIEND}}',    {{all_scopes}}, '2030-08-18 15:48:58.435729+00'),
       (54, {{user_id::ENEMY}},     'enemy-pat',     '{{pat::ENEMY}}',     {{all_scopes}}, '2030-08-18 15:48:58.435729+00');

INSERT INTO loaders (id, loader) VALUES (5, 'fabric');
INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id) VALUES (5, 1);

INSERT INTO loaders (id, loader) VALUES (6, 'forge');
INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id) VALUES (6, 1);

INSERT INTO loaders (id, loader, metadata) VALUES (7, 'bukkit', '{"platform":false}'::JSONB);
INSERT INTO loaders (id, loader, metadata) VALUES (8, 'waterfall', '{"platform":true}'::JSONB);

-- Adds dummies to mrpack_loaders
INSERT INTO loader_field_enum_values (enum_id, value)
SELECT id, 'fabric' FROM loader_field_enums WHERE enum_name = 'mrpack_loaders';
INSERT INTO loader_field_enum_values (enum_id, value)
SELECT id, 'forge' FROM loader_field_enums WHERE enum_name = 'mrpack_loaders';

INSERT INTO loaders_project_types_games (loader_id, project_type_id, game_id)
SELECT joining_loader_id, joining_project_type_id, 1
FROM loaders_project_types
WHERE joining_loader_id = 5;
INSERT INTO loaders_project_types_games (loader_id, project_type_id, game_id)
SELECT joining_loader_id, joining_project_type_id, 1
FROM loaders_project_types
WHERE joining_loader_id = 6;

-- Dummy-data only optional field, as we don't have any yet
INSERT INTO loader_fields
	(field, field_type, optional)
VALUES
	('test_fabric_optional', 'integer', TRUE);
INSERT INTO loader_fields_loaders (loader_id, loader_field_id)
SELECT l.id, lf.id
FROM
	loaders AS l
	CROSS JOIN loader_fields AS lf
WHERE lf.field = 'test_fabric_optional' AND l.loader = 'fabric'
ON CONFLICT DO NOTHING;

-- Sample game versions, loaders, categories
-- Game versions is '2'
INSERT INTO loader_field_enum_values (enum_id, value, metadata, created)
VALUES (2, '1.20.1', '{"type":"release","major":false}', '2021-08-18 15:48:58.435729+00');
INSERT INTO loader_field_enum_values (enum_id, value, metadata, created)
VALUES (2, '1.20.2', '{"type":"release","major":false}', '2021-08-18 15:48:59.435729+00');
INSERT INTO loader_field_enum_values (enum_id, value, metadata, created)
VALUES (2, '1.20.3', '{"type":"release","major":false}', '2021-08-18 15:49:00.435729+00');
INSERT INTO loader_field_enum_values (enum_id, value, metadata, created)
VALUES (2, '1.20.4', '{"type":"beta","major":false}', '2021-08-18 15:49:01.435729+00');
INSERT INTO loader_field_enum_values (enum_id, value, metadata, created)
VALUES (2, '1.20.5', '{"type":"release","major":true}', '2061-08-18 15:49:02.435729+00');

-- Also add 'Ordering_Negative1' and 'Ordering_Positive100' to game versions (to test ordering override)
INSERT INTO loader_field_enum_values (enum_id, value, metadata, ordering)
VALUES (2, 'Ordering_Negative1', '{"type":"release","major":false}', -1);
INSERT INTO loader_field_enum_values (enum_id, value, metadata, ordering)
VALUES (2, 'Ordering_Positive100', '{"type":"release","major":false}', 100);

INSERT INTO loader_fields_loaders (loader_id, loader_field_id)
SELECT l.id, lf.id
FROM
	loaders AS l
	CROSS JOIN loader_fields AS lf
WHERE lf.field IN ('game_versions', 'environment')
ON CONFLICT DO NOTHING;

INSERT INTO categories (id, category, project_type)
VALUES
	(51, 'combat', 1),
	(52, 'decoration', 1),
	(53, 'economy', 1),
	(54, 'food', 1),
	(55, 'magic', 1),
	(56, 'mobs', 1),
	(57, 'optimization', 1);

INSERT INTO categories (id, category, project_type)
VALUES
	(101, 'combat', 2),
	(102, 'decoration', 2),
	(103, 'economy', 2),
	(104, 'food', 2),
	(105, 'magic', 2),
	(106, 'mobs', 2),
	(107, 'optimization', 2);

-- Create dummy oauth client, secret_hash is SHA512 hash of full lowercase alphabet
INSERT INTO oauth_clients (
        id,
        name,
        icon_url,
        max_scopes,
        secret_hash,
        created_by
    )
VALUES (
        1,
        'oauth_client_alpha',
        NULL,
        {{all_scopes}},
        '4dbff86cc2ca1bae1e16468a05cb9881c97f1753bce3619034898faa1aabe429955a1bf8ec483d7421fe3c1646613a59ed5441fb0f321389f77f48a879c7b1f1',
        3
    );
INSERT INTO oauth_client_redirect_uris (id, client_id, uri)
VALUES (1, 1, 'https://modrinth.com/oauth_callback');

-- Create dummy data table to mark that this file has been run
CREATE TABLE dummy_data (
	update_id BIGINT PRIMARY KEY
);
