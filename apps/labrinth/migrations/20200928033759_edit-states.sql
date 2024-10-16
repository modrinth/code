-- Add migration script here
ALTER TABLE states

ADD COLUMN expires timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP + interval '1 hour';