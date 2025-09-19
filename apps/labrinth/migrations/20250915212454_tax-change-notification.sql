ALTER TABLE charges ADD COLUMN tax_last_updated TIMESTAMPTZ;
ALTER TABLE charges ADD COLUMN tax_drift_loss BIGINT;

INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('tax_notification', 2, FALSE, FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'tax_notification', TRUE);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'tax_notification',
		'Your subscription''s tax is changing',
		'https://modrinth.com/email/subscription-tax-change',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'Your {taxnotification.service} subscription''s tax rate is changing. Starting with your next {taxnotification.billing_interval} payment, your {taxnotification.service} subscription''s, your charge and all future charges will be updated as follows:',
			CHR(10),
			CHR(10),
			'Current subtotal: {taxnotification.old_amount}',
			CHR(10),
			'Current tax: {taxnotification.old_tax_amount}',
			CHR(10),
			'Current TOTAL: {taxnotification.old_total_amount}',
			CHR(10),
			CHR(10),
			'New subtotal: {taxnotification.new_amount}',
			CHR(10),
			'New tax: {taxnotification.new_tax_amount}',
			CHR(10),
			'New TOTAL: {taxnotification.new_total_amount}',
			CHR(10),
			CHR(10),
			'Note that the pre-tax price of your subscription has not changed, only the tax charged has changed as required by local tax regulations.',
			CHR(10),
			CHR(10),
			'Thank your for using {taxnotification.service}.'
		)
	);
