ALTER TABLE users
    ADD COLUMN trolley_id text NULL,
    ADD COLUMN trolley_account_status text NULL,
    DROP COLUMN midas_expires,
    DROP COLUMN is_overdue,
    DROP COLUMN stripe_customer_id,
    DROP COLUMN payout_wallet,
    DROP COLUMN payout_wallet_type,
    DROP COLUMN payout_address;

ALTER TABLE historical_payouts
    ADD COLUMN batch_id text NULL,
    ADD COLUMN payment_id text NULL;

UPDATE historical_payouts
SET status = 'processed'