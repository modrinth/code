ALTER TABLE users DROP COLUMN IF EXISTS paypal_email;

ALTER TABLE users
    ADD COLUMN paypal_country text NULL,
    ADD COLUMN paypal_email text NULL,
    ADD COLUMN paypal_id text NULL,
    ADD COLUMN venmo_handle text NULL,

    DROP COLUMN midas_expires,
    DROP COLUMN is_overdue,
    DROP COLUMN stripe_customer_id,
    DROP COLUMN payout_wallet,
    DROP COLUMN payout_wallet_type,
    DROP COLUMN payout_address;

ALTER TABLE historical_payouts
    RENAME TO payouts;

ALTER TABLE payouts
    ADD COLUMN method text NULL,
    ADD COLUMN method_address text NULL,
    ADD COLUMN platform_id text NULL,
    ADD COLUMN fee numeric(40, 20) NULL,
    ALTER COLUMN id TYPE bigint,
    ALTER COLUMN id DROP DEFAULT;

UPDATE payouts
SET status = 'success';

DROP SEQUENCE IF EXISTS historical_payouts_id_seq;
