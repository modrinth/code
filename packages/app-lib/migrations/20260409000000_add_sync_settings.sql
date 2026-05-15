ALTER TABLE settings ADD COLUMN sync_enabled INTEGER NOT NULL DEFAULT 1;
ALTER TABLE settings ADD COLUMN sync_files JSON NOT NULL DEFAULT '["options.txt","servers.dat"]';
ALTER TABLE settings ADD COLUMN sync_folders JSON NOT NULL DEFAULT '["saves","screenshots","resourcepacks","shaderpacks","schematics"]';
