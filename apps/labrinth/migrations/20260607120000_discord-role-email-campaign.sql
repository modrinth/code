INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES
	('discord_role_creator_club', 3, FALSE, FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES
	(NULL, 'email', 'discord_role_creator_club', TRUE);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'discord_role_creator_club',
		'You''re invited to the Creator Club',
		'https://modrinth.com/_internal/templates/email/discord-role-creator-club',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'Thanks for building on Modrinth. Your projects have passed 20,000 total downloads, which is wild to think about.',
			CHR(10),
			CHR(10),
			'That means thousands of players have found something useful, fun, or worth coming back to because of what you made.',
			CHR(10),
			CHR(10),
			'We''re opening up a Creator Club role in the Modrinth Discord for creators like you. Link your Discord account through Modrinth and we''ll sync it automatically.',
			CHR(10),
			CHR(10),
			'Join the Creator Club: {discord.link_url}',
			CHR(10),
			CHR(10),
			'Thanks for making Modrinth what it is,',
			CHR(10),
			'The Modrinth Team'
		)
	);
