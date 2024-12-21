UPDATE dependencies AS d
SET mod_dependency_id = v.mod_id
FROM versions AS v
WHERE v.id = d.dependency_id;

ALTER TABLE users DROP COLUMN flame_anvil_key;
ALTER TABLE mods DROP COLUMN flame_anvil_project;
ALTER TABLE mods DROP COLUMN flame_anvil_user;

ALTER TABLE mods ADD COLUMN monetization_status varchar(64) NOT NULL default 'monetized';