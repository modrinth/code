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
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('project_update', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('team_invite', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('organization_invite', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('status_change', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('moderator_message', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('legacy_markdown', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('unknown', 1, TRUE, TRUE);
INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('reset_password', 3, FALSE, FALSE);