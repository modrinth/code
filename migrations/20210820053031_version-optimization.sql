ALTER TABLE versions ADD COLUMN version_type varchar(255) default 'release' NOT NULL;

UPDATE versions SET version_type = (SELECT rc.channel FROM release_channels rc WHERE rc.id = release_channel);

ALTER TABLE versions DROP COLUMN release_channel, ALTER COLUMN version_type DROP DEFAULT;

DROP TABLE release_channels;