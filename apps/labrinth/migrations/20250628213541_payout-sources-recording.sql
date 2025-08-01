-- Add migration script here
CREATE TABLE payout_sources_balance (
    account_type TEXT NOT NULL,
    amount numeric(40, 20) NOT NULL,
    pending BOOLEAN NOT NULL,
    recorded timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (recorded, account_type, pending)
);
