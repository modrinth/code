ALTER TABLE users ADD COLUMN stripe_customer_id TEXT NULL;

CREATE TABLE products (
    id bigint PRIMARY KEY,
    metadata jsonb NOT NULL,
    unitary BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE products_prices (
    id bigint PRIMARY KEY,
    product_id bigint REFERENCES products NOT NULL,
    currency_code text not null,
    prices jsonb NOT NULL
);

CREATE TABLE users_subscriptions (
    id bigint PRIMARY KEY,
    user_id bigint REFERENCES users NOT NULL,
    price_id bigint REFERENCES products_prices NOT NULL,
    interval text NOT NULL,
    created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_charge timestamptz NULL,
    status varchar(255) NOT NULL
);

CREATE UNIQUE INDEX users_stripe_customer_id
    ON users (stripe_customer_id);

CREATE INDEX products_prices_product
    ON products_prices (product_id);

CREATE INDEX users_subscriptions_users
    ON users_subscriptions (user_id);
