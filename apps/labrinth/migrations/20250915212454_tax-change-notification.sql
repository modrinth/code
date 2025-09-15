INSERT INTO notifications_types (name, delivery_priority, expose_in_user_preferences, expose_in_site_notifications) VALUES ('tax_notification', 2, FALSE, FALSE);

INSERT INTO users_notifications_preferences (user_id, channel, notification_type, enabled)
VALUES (NULL, 'email', 'tax_notification', TRUE);

INSERT INTO notifications_templates (channel, notification_type, subject_line, body_fetch_url, plaintext_fallback)
VALUES (
    'email', 'tax_notification', 'TBD', 'https://modrinth.com/email/subscription-tax-change',
    CONCAT(
        'TBD',
        CHR(10),
        'TBD'
    )
);
