CREATE TABLE users_subscriptions_credits (
	id SERIAL PRIMARY KEY,
	subscription_id BIGINT NOT NULL REFERENCES users_subscriptions (id),
	user_id BIGINT NOT NULL REFERENCES users (id),
	creditor_id BIGINT NOT NULL REFERENCES users (id),
	days INTEGER NOT NULL,
	previous_due TIMESTAMPTZ NOT NULL,
	next_due TIMESTAMPTZ NOT NULL,
	created TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('subscription_credited', 1, FALSE, FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'subscription_credited', TRUE);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'subscription_credited',
		'We’ve added time to your server',
		'https://modrinth.com/_internal/templates/email/subscription-credited',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'{credit.header_message}',
			CHR(10),
			CHR(10),
			'To make up for it, we''ve added {credit.days_formatted} to your {credit.subscription.type} subscription.',
			CHR(10),
			CHR(10),
			'Your next charge was scheduled for {credit.previous_due} and will now be on {credit.next_due}.',
			CHR(10),
			CHR(10),
			'Thank you for supporting us,',
			CHR(10),
			'The Modrinth Team'
		)
	);
