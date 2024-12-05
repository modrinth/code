ALTER TABLE charges
    ADD COLUMN payment_platform TEXT NOT NULL DEFAULT 'stripe',
    ADD COLUMN payment_platform_id TEXT NULL,
    ADD COLUMN net bigint not null DEFAULT 0,
    ADD COLUMN refunded bigint not null DEFAULT 0;