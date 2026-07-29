CREATE INDEX users_lowercase_username_pattern
ON users (LOWER(username) text_pattern_ops);
