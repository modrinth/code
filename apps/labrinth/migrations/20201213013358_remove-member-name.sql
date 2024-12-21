-- Add migration script here
ALTER TABLE team_members
    DROP COLUMN member_name;

UPDATE side_types SET name = 'optional' WHERE name = 'no-functionality';