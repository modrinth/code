ALTER TABLE threads DROP COLUMN show_in_mod_inbox;

ALTER TABLE threads_messages ADD COLUMN hide_identity BOOLEAN default false NOT NULL;

UPDATE threads_messages
SET hide_identity = TRUE
FROM users
WHERE threads_messages.author_id = users.id
AND users.role IN ('moderator', 'admin');