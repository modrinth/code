ALTER TABLE team_members DROP COLUMN payouts_split;
ALTER TABLE team_members ADD COLUMN payouts_split numeric(96, 48) NOT NULL DEFAULT 0;

UPDATE team_members
SET payouts_split = 100
WHERE role = 'Owner';

CREATE TABLE payouts_values (
    id bigserial PRIMARY KEY,
    user_id bigint REFERENCES users NOT NULL,
    mod_id bigint REFERENCES mods NULL,
    amount numeric(96, 48) NOT NULL,
    created timestamptz NOT NULL,
    claimed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX payouts_values_user_id
    ON payouts_values (user_id);

CREATE INDEX payouts_values_mod_id
    ON payouts_values (mod_id);

CREATE INDEX payouts_values_created
    ON payouts_values (created);

ALTER TABLE users ADD COLUMN midas_expires timestamptz NULL;
ALTER TABLE users ADD COLUMN is_overdue BOOLEAN NULL;
ALTER TABLE users ADD COLUMN stripe_customer_id varchar(255) NULL;
ALTER TABLE users ADD COLUMN paypal_email varchar(128) NULL;