CREATE TABLE notifications_deliveries (
    id BIGSERIAL PRIMARY KEY,
    notification_id BIGINT NOT NULL REFERENCES notifications(id),
    channel VARCHAR(32) NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users(id),
    delivery_priority INTEGER NOT NULL,
    status VARCHAR(32) NOT NULL,
    next_attempt TIMESTAMP WITH TIME ZONE NOT NULL,
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

CREATE TABLE notifications_types_preference_restrictions (
    notification_type VARCHAR(32) NOT NULL REFERENCES notifications_types(name),
    channel VARCHAR(32) NOT NULL,
    forced_value BOOL NOT NULL
);

CREATE TABLE notifications_templates (
    id BIGSERIAL PRIMARY KEY,
    channel VARCHAR(32) NOT NULL,
    notification_type VARCHAR(32) NOT NULL REFERENCES notifications_types(name),
    subject_line TEXT NOT NULL,
    body_fetch_url TEXT NOT NULL,
    plaintext_fallback TEXT NOT NULL
);

-- Add existing notification types
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('reset_password', 3, FALSE, FALSE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('project_update', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('team_invite', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('organization_invite', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('status_change', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('moderator_message', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('legacy_markdown', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('unknown', 1, TRUE, TRUE);

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

-- Pre-insert any templates
INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'reset_password', 'Reset your Modrinth password', 'https://modrinth.com/mail/resetpassword.html',
    CONCAT(
        'Hi {user.name},',
        CHR(10),
        CHR(10),
        'Please visit the link below to reset your password. If you did not request for your password to be reset, you can safely ignore this email.',
        CHR(10),
        'Reset your password: {resetpassword.url}',
        CHR(10),
        CHR(10),
        '- The Modrinth Team',
        CHR(10),
        'modrinth.com'
    )
);