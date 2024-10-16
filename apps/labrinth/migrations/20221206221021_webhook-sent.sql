-- Add migration script here
ALTER TABLE mods ADD COLUMN webhook_sent BOOL NOT NULL DEFAULT FALSE;

UPDATE mods
SET webhook_sent = (status = 'approved');

UPDATE mods
SET status = 'withheld'
WHERE status = 'unlisted';