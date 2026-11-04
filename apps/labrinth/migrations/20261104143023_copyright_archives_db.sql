CREATE TABLE copyright_archives (
  id   BIGINT PRIMARY KEY,
  meta JSONB  NOT NULL
);

CREATE TABLE copyright_archive_files (
  archive_id BIGINT NOT NULL REFERENCES copyright_archives(id) ON DELETE CASCADE,
  path       TEXT   NOT NULL,
  sha512     BYTEA  NOT NULL,
  PRIMARY KEY (archive_id, path)
);
