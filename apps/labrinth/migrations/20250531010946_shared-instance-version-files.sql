ALTER TABLE shared_instance_versions
ADD COLUMN size BIGINT NOT NULL DEFAULT 0,
ADD COLUMN sha512 bytea NOT NULL DEFAULT E'';

UPDATE shared_instance_versions version
SET
  size = file.size,
  sha512 = decode(encode(hash.hash, 'escape'), 'hex')
FROM files file
JOIN hashes hash ON file.id = hash.file_id AND hash.algorithm = 'sha512'
WHERE file.id = version.file_id;

ALTER TABLE shared_instance_versions
DROP COLUMN file_id,
ALTER COLUMN size DROP DEFAULT,
ALTER COLUMN sha512 DROP DEFAULT;
