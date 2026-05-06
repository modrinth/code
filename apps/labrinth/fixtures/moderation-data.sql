-- Dummy moderation_external_licenses (explicit IDs required)
INSERT INTO moderation_external_licenses (id, title, status, link, exceptions, proof, flame_project_id, inserted_at, inserted_by, updated_at, updated_by)
VALUES
    (9001, 'Example Mod',        'yes',              'https://example.com/license',  NULL,                       'Verified by team',      101,  now(), 0, now(), 0),
    (9002, 'Cool Resource Pack', 'no',               'https://coolpack.com/terms',   'Non-commercial only',      'DMCA takedown filed',   202,  now(), 0, now(), 0),
    (9003, 'Mystery Project',    'unidentified',     NULL,                           NULL,                       NULL,                    NULL, now(), 0, now(), 0),
    (9004, 'Widget Lib',         'with-attribution', 'https://widgets.dev/MIT',      NULL,                       'License header in JAR', 303,  now(), 0, now(), 0),
    (9005, 'Shadow Mod',         'permanent-no',     'https://shadow.net/eula',      'Redistribution restricted','Under review',          NULL, now(), 0, now(), 0);

-- Dummy moderation_external_files (sha1 stored as raw binary bytea via decode())
INSERT INTO moderation_external_files (sha1, filename, external_license_id)
VALUES
    (decode('aabbccdd11223344aabbccdd11223344aabbccdd', 'hex'), 'example-mod-1.0.jar', 9001),
    (decode('11223344aabbccdd11223344aabbccdd11223344', 'hex'), 'example-mod-1.1.jar', 9001),
    (decode('deadbeefdeadbeefdeadbeefdeadbeefdeadbeef', 'hex'), 'coolpack-v2.zip',     9002),
    (decode('cafebabecafebabecafebabecafebabecafebabe', 'hex'), 'mystery.dat',         9003),
    (decode('0102030405060708090a0b0c0d0e0f1011121314', 'hex'), 'widget-lib.jar',      9004);
-- License 9005 intentionally has no files (tests empty linked_files case)
