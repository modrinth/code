ALTER TABLE users
    ADD COLUMN webauthn_passkeys jsonb not null default '{}' -- Empty map