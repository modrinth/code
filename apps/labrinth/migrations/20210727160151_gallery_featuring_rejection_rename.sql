ALTER TABLE mods
    RENAME COLUMN rejection_reason TO moderation_message;
ALTER TABLE mods
    RENAME COLUMN rejection_body TO moderation_message_body;

ALTER TABLE mods_gallery
    ADD COLUMN featured boolean default false;