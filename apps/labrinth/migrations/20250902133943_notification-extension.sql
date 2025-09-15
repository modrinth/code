CREATE TABLE notifications_deliveries (
    id BIGSERIAL PRIMARY KEY,
    notification_id BIGINT NOT NULL REFERENCES notifications(id),
    channel VARCHAR(32) NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users(id),
    delivery_priority INTEGER NOT NULL,
    status VARCHAR(32) NOT NULL,
    next_attempt timestamptz NOT NULL,
    attempt_count INTEGER NOT NULL,

    UNIQUE (notification_id, channel)
);

CREATE INDEX idx_notifications_deliveries_composite_queue
ON notifications_deliveries(channel, status, next_attempt ASC, delivery_priority DESC)
INCLUDE (notification_id, user_id);

CREATE INDEX idx_notifications_deliveries_user_id
ON notifications_deliveries(user_id);

CREATE TABLE users_notifications_preferences (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES users(id),
    channel VARCHAR(32) NOT NULL,
    notification_type VARCHAR(32) NOT NULL,
    enabled BOOL NOT NULL
);

CREATE INDEX idx_users_notifications_preferences_user_id
ON users_notifications_preferences(user_id);

CREATE UNIQUE INDEX idx_users_notifications_preferences_partial_contextual_uniq
ON users_notifications_preferences(COALESCE(user_id, -1), channel, notification_type);

CREATE TABLE notifications_types (
    name VARCHAR(32) PRIMARY KEY,
    delivery_priority INTEGER NOT NULL,
    expose_in_user_preferences BOOL NOT NULL,
    expose_in_site_notifications BOOL NOT NULL
);

CREATE TABLE notifications_templates (
    id BIGSERIAL PRIMARY KEY,
    channel VARCHAR(32) NOT NULL,
    notification_type VARCHAR(32) NOT NULL REFERENCES notifications_types(name),
    subject_line TEXT NOT NULL,
    body_fetch_url TEXT NOT NULL,
    plaintext_fallback TEXT NOT NULL
);

INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('reset_password', 3, FALSE, FALSE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('project_update', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('team_invite', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('organization_invite', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('status_change', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('moderator_message', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('legacy_markdown', 1, FALSE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('unknown', 1, FALSE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('verify_email', 3, FALSE, FALSE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('auth_provider_added', 2, FALSE, FALSE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('auth_provider_removed', 2, FALSE, FALSE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('two_factor_enabled', 2, FALSE, FALSE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('two_factor_removed', 2, FALSE, FALSE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('password_changed', 2, FALSE, FALSE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('password_removed', 2, FALSE, FALSE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('email_changed', 2, FALSE, FALSE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('payment_failed', 2, FALSE, FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'reset_password', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'project_update', FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'team_invite', FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'organization_invite', FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'status_change', FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'moderator_message', FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'legacy_markdown', FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'unknown', FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'verify_email', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'auth_provider_added', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'auth_provider_removed', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'two_factor_enabled', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'two_factor_removed', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'password_changed', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'password_removed', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'email_changed', TRUE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'payment_failed', TRUE);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'reset_password', 'Reset your Modrinth password', 'https://modrinth.com/email/reset-password',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'Please visit the link below to reset your password. If you did not request for your password to be reset, you can safely ignore this email.',
        CHR(10),
        'Reset your password: {resetpassword.url}'
    )
);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'verify_email', 'Verify your Modrinth email', 'https://modrinth.com/email/verify-email',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'Please visit the link below to verify your Modrinth email. If the button does not work, you can copy the link and paste it into your browser. This link expires in 24 hours.',
        CHR(10),
        'Verify your email: {verifyemail.url}'
    )
);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'auth_provider_added', 'Authentication method added', 'https://modrinth.com/email/auth-provider-added',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'When logging into Modrinth, you can now log in using the ', '{authprovider.name}', ' authentication provider.',
        CHR(10),
        'If you did not make this change, please contact us immediately by replying to this email or through our support portal at https://support.modrinth.com (using',
        'the green chat bubble at the bottom of the page)'
    )
);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'auth_provider_removed', 'Authentication method removed', 'https://modrinth.com/email/auth-provider-removed',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'When logging into Modrinth, you can no longer log in using the ', '{authprovider.name}', ' authentication provider.',
        CHR(10),
        'If you did not make this change, please contact us immediately by replying to this email or through our support portal at https://support.modrinth.com (using',
        'the green chat bubble at the bottom of the page)'
    )
);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'two_factor_enabled', 'Two-factor authentication enabled', 'https://modrinth.com/email/two-factor-enabled',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'When logging into Modrinth, you can now enter a code generated by your authenticator app in addition to entering your usual email address and password.',
        CHR(10),
        'If you did not make this change, please contact us immediately by replying to this email or through our support portal at https://support.modrinth.com (using',
        'the green chat bubble at the bottom of the page)'
    )
);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'two_factor_removed', 'Two-factor authentication removed', 'https://modrinth.com/email/two-factor-removed',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'When logging into Modrinth, you no longer need two-factor authentication to gain access.',
        CHR(10),
        'If you did not make this change, please contact us immediately by replying to this email or through our support portal at https://support.modrinth.com (using',
        'the green chat bubble at the bottom of the page)'
    )
);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'password_changed', 'Your Modrinth password was changed', 'https://modrinth.com/email/password-changed',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'Your password has been changed on your account.',
        CHR(10),
        'If you did not make this change, please contact us immediately by replying to this email or through our support portal at https://support.modrinth.com (using',
        'the green chat bubble at the bottom of the page)'
    )
);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'password_removed', 'Your Modrinth password was removed', 'https://modrinth.com/email/password-removed',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'Your password has been removed on your account.',
        CHR(10),
        'If you did not make this change, please contact us immediately by replying to this email or through our support portal at https://support.modrinth.com (using',
        'the green chat bubble at the bottom of the page)'
    )
);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'email_changed', 'Your Modrinth email was changed', 'https://modrinth.com/email/email-changed',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'Your Modrinth account email has been updated to {emailchanged.new_email}.',
        CHR(10),
        'If you did not make this change, please contact us immediately by replying to this email or through our support portal at https://support.modrinth.com (using',
        'the green chat bubble at the bottom of the page)'
    )
);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'payment_failed', 'Payment Failed for Modrinth', 'https://modrinth.com/email/payment-failed',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'Our attempt to collect payment for {paymentfailed.amount} from the payment card on file was unsuccessful. Please update your billing settings to avoid service termination.',
        CHR(10),
        'Update billing settings: {billing.url}'
    )
);