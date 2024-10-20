ALTER TABLE threads ADD COLUMN show_in_mod_inbox BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE threads_messages DROP COLUMN show_in_mod_inbox;

ALTER TABLE notifications ADD COLUMN body jsonb NULL;
ALTER TABLE notifications ALTER COLUMN title DROP NOT NULL;
ALTER TABLE notifications ALTER COLUMN text DROP NOT NULL;
ALTER TABLE notifications ALTER COLUMN link DROP NOT NULL;

ALTER TABLE threads ADD COLUMN report_id bigint REFERENCES reports ON UPDATE CASCADE;
ALTER TABLE threads ADD COLUMN project_id bigint REFERENCES mods ON UPDATE CASCADE;