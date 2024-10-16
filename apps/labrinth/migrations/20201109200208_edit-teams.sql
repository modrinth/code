-- Add migration script here
ALTER TABLE team_members
ADD COLUMN permissions bigint default 0 NOT NULL;
ALTER TABLE team_members
ADD COLUMN accepted boolean default false NOT NULL;