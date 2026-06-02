INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('server_invite', 1, FALSE, TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'server_invite', FALSE);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'server_invite',
		'You''ve been invited to a server',
		'https://modrinth.com/_internal/templates/email/server-invited',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'Modrinth user {inviter.name} has invited you to help manage {server.name} on Modrinth Hosting with the {server.role} role.',
			CHR(10),
			CHR(10),
			'To accept or reject this invitation, open your Modrinth notifications: https://modrinth.com/dashboard/notifications',
			CHR(10),
			CHR(10),
			'If you were not expecting this invitation, contact the server owner or reach out to Modrinth Support at https://support.modrinth.com'
		)
	);
