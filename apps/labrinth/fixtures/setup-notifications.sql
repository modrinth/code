INSERT INTO notifications_types (name, delivery_priority, always_enabled) VALUES ('team_invite', 3, FALSE);

INSERT INTO users_notifications_preferences (id, user_id, channel, notification_type, enabled)
VALUES (RANDOM(1000000000, 10000000000), NULL, 'email', 'team_invite', FALSE);

INSERT INTO notifications_templates
(id, channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (RANDOM(10000000, 100000000), 'email', 'team_invite', 'Invited to a team', 'https://modrinth.com/mail/teaminvite.html', 'Hello {user.name}. You have been invited to {teaminvite.project.name} for the role of {teaminvite.role.name} by {teaminvite.inviter.name}.');
