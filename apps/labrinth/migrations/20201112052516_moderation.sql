-- Add migration script here
DELETE FROM release_channels WHERE channel = 'release-hidden';
DELETE FROM release_channels WHERE channel = 'beta-hidden';
DELETE FROM release_channels WHERE channel = 'alpha-hidden';

ALTER TABLE versions
ADD COLUMN accepted BOOLEAN NOT NULL default FALSE;