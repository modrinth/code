ALTER TABLE version_fields
DROP CONSTRAINT version_fields_enum_value_fkey;

ALTER TABLE version_fields
    ALTER COLUMN enum_value SET DEFAULT -1;

UPDATE version_fields SET enum_value = -1 WHERE enum_value IS NULL;

ALTER TABLE version_fields
    ALTER COLUMN enum_value SET NOT NULL;

WITH CTE AS (
    SELECT ctid,
           ROW_NUMBER() OVER (PARTITION BY version_id, field_id, enum_value ORDER BY ctid) AS row_num
    FROM version_fields
)
DELETE FROM version_fields
WHERE ctid IN (
    SELECT ctid
    FROM CTE
    WHERE row_num > 1
);

ALTER TABLE version_fields
ADD PRIMARY KEY (version_id, field_id, enum_value);

ALTER TABLE loader_fields_loaders
ADD PRIMARY KEY (loader_id, loader_field_id);