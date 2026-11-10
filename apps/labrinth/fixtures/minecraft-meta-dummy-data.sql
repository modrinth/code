-- Metadata-only catalog for isolated Labrinth test databases, not playable manifests.
CREATE TABLE IF NOT EXISTS minecraft_versions (
    name TEXT PRIMARY KEY,
    catalog_entry JSON NOT NULL,
    manifest JSON NOT NULL,
    source_catalog_sha256 TEXT NOT NULL,
    source_manifest_sha256 TEXT NOT NULL,
    is_latest_release BOOLEAN,
    is_latest_snapshot BOOLEAN
);

CREATE UNIQUE INDEX IF NOT EXISTS index_minecraft_versions_by_is_latest_release
    ON minecraft_versions (is_latest_release);
CREATE UNIQUE INDEX IF NOT EXISTS index_minecraft_versions_by_is_latest_snapshot
    ON minecraft_versions (is_latest_snapshot);

INSERT INTO minecraft_versions (
    name, catalog_entry, manifest, source_catalog_sha256, source_manifest_sha256,
    is_latest_release, is_latest_snapshot
)
SELECT
    name,
    json_build_object(
        'id', name, 'type', version_type,
        'url', 'https://metadata-only.invalid/manifest',
        'time', released_at, 'releaseTime', released_at,
        'sha1', repeat('0', 40), 'complianceLevel', 0
    ),
    json_build_object(
        'id', name, 'type', version_type, 'time', released_at, 'releaseTime', released_at,
        'assetIndex', json_build_object(
            'id', '', 'sha1', repeat('0', 40), 'size', 0, 'totalSize', 0,
            'url', 'https://metadata-only.invalid/assets'
        ),
        'assets', '', 'downloads', '{}'::json, 'libraries', '[]'::json,
        'mainClass', '', 'minimumLauncherVersion', 0
    ),
    repeat('0', 64), repeat('0', 64), NULL::boolean, NULL::boolean
FROM (VALUES
    ('1.3', 'snapshot', '2012-07-25T22:00:00Z'),
    ('1.13.2', 'release', '2018-10-22T11:41:07Z'),
    ('18w43b', 'snapshot', '2018-10-24T15:02:30Z'),
    ('1.20.1', 'release', '2021-08-18T15:48:58.435729Z'),
    ('1.20.2', 'release', '2021-08-18T15:48:59.435729Z'),
    ('1.20.3', 'release', '2021-08-18T15:49:00.435729Z'),
    ('1.20.4', 'old_beta', '2021-08-18T15:49:01.435729Z'),
    ('1.20.5', 'release', '2061-08-18T15:49:02.435729Z')
) AS versions(name, version_type, released_at)
ON CONFLICT (name) DO NOTHING;
