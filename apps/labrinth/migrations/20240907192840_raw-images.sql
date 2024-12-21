ALTER TABLE mods ADD COLUMN raw_icon_url TEXT NULL;
UPDATE mods SET raw_icon_url = icon_url;

ALTER TABLE users ADD COLUMN raw_avatar_url TEXT NULL;
UPDATE users SET raw_avatar_url = avatar_url;

ALTER TABLE oauth_clients ADD COLUMN raw_icon_url TEXT NULL;
UPDATE oauth_clients SET raw_icon_url = icon_url;

ALTER TABLE organizations ADD COLUMN raw_icon_url TEXT NULL;
UPDATE organizations SET raw_icon_url = icon_url;

ALTER TABLE collections ADD COLUMN raw_icon_url TEXT NULL;
UPDATE collections SET raw_icon_url = icon_url;

ALTER TABLE mods_gallery ADD COLUMN raw_image_url TEXT NULL;
UPDATE mods_gallery SET raw_image_url = image_url;
ALTER TABLE mods_gallery ALTER COLUMN raw_image_url SET NOT NULL;

ALTER TABLE uploaded_images ADD COLUMN raw_url TEXT NULL;
UPDATE uploaded_images SET raw_url = url;
ALTER TABLE uploaded_images ALTER COLUMN raw_url SET NOT NULL;