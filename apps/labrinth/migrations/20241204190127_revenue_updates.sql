ALTER TABLE charges
    ADD COLUMN payment_platform TEXT NOT NULL DEFAULT 'stripe',
    ADD COLUMN payment_platform_id TEXT NULL,
    ADD COLUMN parent_charge_id BIGINT REFERENCES charges(id) NULL,
    ADD COLUMN net BIGINT NULL;