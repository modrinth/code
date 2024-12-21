ALTER TABLE users DROP COLUMN paypal_email;
ALTER TABLE payouts_values DROP COLUMN claimed;

ALTER TABLE users ADD COLUMN payout_wallet varchar(128) NULL;
ALTER TABLE users ADD COLUMN payout_wallet_type varchar(128) NULL;
ALTER TABLE users ADD COLUMN payout_address varchar(128) NULL;
ALTER TABLE users ADD COLUMN balance numeric(96, 48) NOT NULL DEFAULT 0;

UPDATE users
SET balance = COALESCE((SELECT SUM(T2.amount) FROM payouts_values T2 WHERE T2.user_id = users.id), 0, 0)
WHERE id > 1;

CREATE TABLE historical_payouts (
    id bigserial PRIMARY KEY,
    user_id bigint REFERENCES users NOT NULL,
    amount numeric(96, 48) NOT NULL,
    created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    status varchar(128) NOT NULL
);

DELETE FROM payouts_values WHERE amount = 0;

CREATE INDEX historical_payouts_user_id
    ON historical_payouts (user_id);