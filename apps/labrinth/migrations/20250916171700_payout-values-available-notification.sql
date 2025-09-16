CREATE TABLE payouts_values_notifications (
	id SERIAL PRIMARY KEY,
	date_available TIMESTAMPTZ NOT NULL,
	user_id BIGINT NOT NULL REFERENCES users (id),
	notified BOOLEAN NOT NULL
);

CREATE UNIQUE INDEX payouts_values_notifications_date_available_user_id_idx ON payouts_values_notifications (
	date_available,
	user_id
);

INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('payout_available', 1, TRUE, FALSE);
INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('pat_created', 3, FALSE, FALSE);
INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('moderation_message_received', 1, TRUE, FALSE);
INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('report_status_updated', 1, TRUE, FALSE);
INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('report_submitted', 1, TRUE, FALSE);
INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('project_status_approved', 1, TRUE, FALSE);
INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('project_status_neutral', 1, TRUE, FALSE);
INSERT INTO notifications_types
	(name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications)
VALUES ('project_transferred', 2, FALSE, FALSE);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'payout_available',
		'Revenue available to withdraw!',
		'https://modrinth.com/email/payout-available',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'The ${payout.amount} earned during {payout.period} has been processed and is now available to withdraw from your account.',
			CHR(10),
			CHR(10),
			'View your revenue dashboard: https://modrinth.com/dashboard/revenue',
			CHR(10),
			CHR(10),
			'If you have any questions about the creator rewards program, please contact support through the Support Portal at https://support.modrinth.com/ or by replying to this email.',
			CHR(10),
			CHR(10),
			'Thank you for being a creator on Modrinth!'
		)
	);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'pat_created',
		'A new personal access token has been created',
		'https://modrinth.com/email/personal-access-token-created',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'A new personal access token, {newpat.token_name}, has been added to your account.',
			CHR(10),
			CHR(10),
			'If you did not create this token, please contact us immediately by replying to this email or through our Support Portal.',
			CHR(10),
			CHR(10),
			'Support Portal: https://support.modrinth.com/'
		)
	);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'moderation_message_received',
		'New message from moderators on your project',
		'https://modrinth.com/email/moderation-thread-message-received',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'Modrinth''s moderation team has left a message on your project, {project.name}.',
			CHR(10),
			CHR(10),
			'Please sign in to view the message and reply if requested. It''s important to address feedback from the moderation team promptly.',
			CHR(10),
			'Your project''s moderation thread: https://modrinth.com/project/{project.id}/moderation',
			CHR(10),
			CHR(10),
			'Thank you for publishing on Modrinth!'
		)
	);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'moderation_message_received',
		'New message from moderators on your project',
		'https://modrinth.com/email/moderation-thread-message-received',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'Modrinth''s moderation team has left a message on your project, {project.name}.',
			CHR(10),
			CHR(10),
			'Please sign in to view the message and reply if requested. It''s important to address feedback from the moderation team promptly.',
			CHR(10),
			'Your project''s moderation thread: https://modrinth.com/project/{project.id}/moderation',
			CHR(10),
			CHR(10),
			'Thank you for publishing on Modrinth!'
		)
	);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'report_status_updated',
		'Your report has been updated',
		'https://modrinth.com/email/report-updated',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'Your report of {report.title} from {report.date} has been updated by our moderation team.',
			CHR(10),
			CHR(10),
			'You can view the full report thread to see the update. If you have more information to add, please reply in the report thread for our moderators to review.',
			CHR(10),
			CHR(10),
			'Thank you for helping keep Modrinth safe and welcoming for everyone.'
		)
	);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'report_submitted',
		'Your report has been submitted',
		'https://modrinth.com/email/report-submitted',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'We''ve received your report of {report.title} and our moderation team will review it shortly.',
			CHR(10),
			CHR(10),
			'Our team takes all reports seriously and will investigate according to our Content Rules, Terms of Service and Copyright Policy. You''ll receive an email update once we''ve completed our review.',
			CHR(10),
			CHR(10),
			'If you have any additional information about this report, you can view it here: https://modrinth.com/dashboard/report/{newreport.id}',
			CHR(10),
			CHR(10),
			'Thank you for helping keep Modrinth safe and welcoming for everyone.'
		)
	);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'project_status_approved',
		'Your project, {project.name}, has been approved 🎉',
		'https://modrinth.com/email/project-approved',
		CONCAT(
			'Congratulations {user.name},',
			CHR(10),
			CHR(10),
			'Your project {project.name} has been approved by the moderation team!',
			CHR(10),
			CHR(10),
			'View your project here: https://modrinth.com/project/{project.id}',
			CHR(10),
			CHR(10),
			'If you have questions or believe something isn''t correct, you can reply to this email or reach out via the Support Portal.',
			CHR(10),
			CHR(10),
			'Thank you for sharing your work with the Modrinth community!'
		)
	);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'project_status_neutral',
		'Your project''s status has been updated',
		'https://modrinth.com/email/project-status-updated-neutral',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'Your project''s status has been changed from {project.oldstatus} to {project.newstatus} by the moderation team. Please review any messages left in the moderation thread which might be relevant to why the status was changed.',
			CHR(10),
			CHR(10),
			'View your project here: https://modrinth.com/project/{project.id}/moderation',
			CHR(10),
			CHR(10),
			'If you believe this status was applied in error, you can reply in the moderation thread or contact support through our Support Portal or by replying to this email.',
			CHR(10),
			CHR(10),
			'Thank you for publishing on Modrinth!'
		)
	);

INSERT INTO notifications_templates
	(channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES
	(
		'email',
		'project_transferred',
		'Project ownership transferred',
		'https://modrinth.com/email/project-ownership-transferred',
		CONCAT(
			'Hi {user.name},',
			CHR(10),
			CHR(10),
			'The ownership of {project.name} has been successfully transferred to the Modrinth {new_owner.type_capitalized} {new_owner.name}.',
			CHR(10),
			CHR(10),
			'View the project here: https://modrinth.com/project/{project.id}',
			CHR(10),
			CHR(10),
			'If you did not initiate this transfer, please contact support immediately through the Support Portal or by replying to this email.'
		)
	);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES
	(NULL, 'email', 'payout_available', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES
	(NULL, 'email', 'pat_created', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES
	(NULL, 'email', 'moderation_message_received', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES
	(NULL, 'email', 'report_status_updated', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES
	(NULL, 'email', 'report_submitted', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES
	(NULL, 'email', 'project_status_approved', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES
	(NULL, 'email', 'project_status_neutral', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES
	(NULL, 'email', 'project_transferred', TRUE);
