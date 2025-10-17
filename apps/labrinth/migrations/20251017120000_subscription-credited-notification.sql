-- Audit table for subscription credits
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

CREATE INDEX users_subscriptions_credits_subscription_created_idx ON users_subscriptions_credits (
	subscription_id,
	created
);

INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('subscription_credited', 2, FALSE, FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'subscription_credited', TRUE);

-- Register email template
INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'subscription_credited',
		'We added {credit.days} days to your subscription',
		'https://modrinth.com/email/subscription-credited',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'We have credited your subscription by {credit.days} days.',
			CHR(10),
			'Previous due date: {credit.previous_due}',
			CHR(10),
			'New due date: {credit.next_due}',
			CHR(10),
			CHR(10),
			'Thank you for using Modrinth.'
		)
	);
