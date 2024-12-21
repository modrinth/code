-- Add migration script here
ALTER TABLE payouts_values ALTER amount TYPE numeric(40, 20);

ALTER TABLE users ADD COLUMN flame_anvil_key varchar(40) NULL;
ALTER TABLE mods ADD COLUMN flame_anvil_project integer NULL;
ALTER TABLE mods ADD COLUMN flame_anvil_user bigint REFERENCES users NULL;
