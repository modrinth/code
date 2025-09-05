INSERT INTO notifications
(id, user_id, name, text, link, created, read, type, body)
VALUES
(RANDOM(100000000,10000000000), (SELECT id FROM users ORDER BY created LIMIT 1), 'notification', '', '', NOW(), FALSE, 'team_invite', JSONB_BUILD_OBJECT(
  'type', 'team_invite',
  'project_id', to_base62((SELECT id FROM mods ORDER BY updated LIMIT 1)),
  'team_id', to_base62((SELECT team_id FROM mods ORDER BY updated LIMIT 1)),
  'invited_by', to_base62((SELECT id FROM users ORDER BY created LIMIT 1)),
  'role', 'developer'
));

INSERT INTO notifications_deliveries
(id, notification_id, channel, user_id, delivery_priority, status, next_attempt, attempt_count)
VALUES (RANDOM(100000000,10000000000), (SELECT id FROM notifications ORDER BY created DESC LIMIT 1), 'email', (SELECT user_id FROM notifications ORDER BY created DESC LIMIT 1), 3, 'pending', NOW(), 0);
