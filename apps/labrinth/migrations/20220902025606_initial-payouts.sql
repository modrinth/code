ALTER TABLE team_members ADD COLUMN payouts_split REAL NOT NULL DEFAULT 0;

UPDATE team_members
SET permissions = 1023, payouts_split = 100
WHERE role = 'Owner';

ALTER TABLE users ADD COLUMN badges bigint default 0 NOT NULL;
