-- rename 'title' to 'name' in all tables (collections, organizations, mods, mods_gallery, notifications, notifications_actions)
ALTER TABLE collections RENAME COLUMN title TO name;
ALTER TABLE organizations RENAME COLUMN title TO name;
ALTER TABLE mods RENAME COLUMN title TO name;
ALTER TABLE mods_gallery RENAME COLUMN title TO name;
ALTER TABLE notifications RENAME COLUMN title TO name;
ALTER TABLE notifications_actions RENAME COLUMN title TO name;

-- rename project 'description' to 'summary'
-- rename project 'body' to 'description'
ALTER TABLE mods RENAME COLUMN description TO summary;
ALTER TABLE mods RENAME COLUMN body TO description;

-- Adds 'is_owner' boolean to team members table- only one can be true.
ALTER TABLE team_members ADD COLUMN is_owner boolean NOT NULL DEFAULT false;
UPDATE team_members SET is_owner = true WHERE role = 'Owner';