ALTER TABLE project_disclosures
	ADD COLUMN lock_status TEXT NOT NULL DEFAULT 'unlocked';

UPDATE project_disclosures
SET lock_status = 'fully_locked'
WHERE set_by_moderator;
