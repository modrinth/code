-- Add migration script here
ALTER TABLE versions
ALTER COLUMN mod_id SET NOT NULL;

ALTER TABLE release_channels
ALTER COLUMN channel SET NOT NULL;
