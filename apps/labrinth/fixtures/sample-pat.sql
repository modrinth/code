INSERT INTO pats
(id, name, user_id, access_token, scopes, created, expires, last_used)
VALUES (RANDOM(100000000,10000000000), 'Sample PAT', (SELECT id FROM users WHERE username = 'Ghost'), 'mrp_abcdefg', 9223372036854775807, NOW(), NOW() + INTERVAL '1 year', NOW());