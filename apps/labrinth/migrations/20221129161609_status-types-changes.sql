-- Add migration script here
ALTER TABLE mods ADD COLUMN updated_status varchar(128) NULL;
ALTER TABLE mods ADD COLUMN requested_status varchar(128) NULL;

UPDATE mods
SET updated_status = (
    SELECT s.status
    FROM statuses s
    WHERE s.id = mods.status
);

ALTER TABLE mods
DROP COLUMN status;

ALTER TABLE mods
RENAME COLUMN updated_status TO status;

DROP TABLE statuses;

ALTER TABLE mods ALTER COLUMN status SET NOT NULL;

ALTER TABlE versions ADD COLUMN status varchar(128) NOT NULL DEFAULT 'listed';
ALTER TABLE versions ADD COLUMN requested_status varchar(128) NULL;