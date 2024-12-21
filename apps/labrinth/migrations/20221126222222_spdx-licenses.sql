ALTER TABLE mods ADD COLUMN license_new varchar(2048) DEFAULT 'LicenseRef-All-Rights-Reserved' NOT NULL;

UPDATE mods SET license_new = licenses.short FROM licenses WHERE mods.license = licenses.id;

UPDATE mods SET license_new = 'LicenseRef-Custom' WHERE license_new = 'custom';
UPDATE mods SET license_new = 'LicenseRef-All-Rights-Reserved' WHERE license_new = 'arr';
UPDATE mods SET license_new = 'Apache-2.0' WHERE license_new = 'apache';
UPDATE mods SET license_new = 'BSD-2-Clause' WHERE license_new = 'bsd-2-clause';
UPDATE mods SET license_new = 'BSD-3-Clause' WHERE license_new = 'bsd-3-clause' OR license_new = 'bsd';
UPDATE mods SET license_new = 'CC0-1.0' WHERE license_new = 'cc0';
UPDATE mods SET license_new = 'Unlicense' WHERE license_new = 'unlicense';
UPDATE mods SET license_new = 'MIT' WHERE license_new = 'mit';
UPDATE mods SET license_new = 'LGPL-3.0-only' WHERE license_new = 'lgpl-3';
UPDATE mods SET license_new = 'LGPL-2.1-only' WHERE license_new = 'lgpl-2.1' OR license_new = 'lgpl';
UPDATE mods SET license_new = 'MPL-2.0' WHERE license_new = 'mpl-2';
UPDATE mods SET license_new = 'ISC' WHERE license_new = 'isc';
UPDATE mods SET license_new = 'Zlib' WHERE license_new = 'zlib';
UPDATE mods SET license_new = 'GPL-2.0-only' WHERE license_new = 'gpl-2';
UPDATE mods SET license_new = 'GPL-3.0-only' WHERE license_new = 'gpl-3';
UPDATE mods SET license_new = 'AGPL-3.0-only' WHERE license_new = 'agpl';

UPDATE mods SET license_url = NULL WHERE license_url LIKE 'https://cdn.modrinth.com/licenses/%';

ALTER TABLE mods DROP COLUMN license;
ALTER TABLE mods RENAME COLUMN license_new TO license;

DROP TABLE licenses;
