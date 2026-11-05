INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('shared_instance_quarantined', 1, FALSE, TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'shared_instance_quarantined', TRUE);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'shared_instance_quarantined',
		'Notice: Your shared instance "{sharedinstance.name}" has been locked',
		'https://modrinth.com/_internal/templates/email/shared-instance-quarantined',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'We''re emailing you to let you know that your shared instance, "{sharedinstance.name}", has been locked by the Modrinth moderation team as it was found to break our Terms of Service (https://modrinth.com/legal/terms).',
			CHR(10),
			CHR(10),
			'While it remains locked, the instance cannot be played or modified, and new members cannot be invited to it.',
			CHR(10),
			CHR(10),
			'{sharedinstance.reason_plaintext}',
			'If you believe this action was taken in error, please contact Modrinth Support (https://support.modrinth.com) and reference the name of the affected instance.'
		)
	);
