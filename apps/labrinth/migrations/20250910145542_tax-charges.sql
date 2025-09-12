ALTER TABLE charges ADD COLUMN tax_amount BIGINT NOT NULL DEFAULT 0;
ALTER TABLE charges ADD COLUMN tax_platform_id TEXT;

ALTER TABLE users_subscriptions ADD COLUMN user_aware_of_tax_changes BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE products_tax_identifiers (
    id SERIAL PRIMARY KEY,
    tax_processor_id TEXT NOT NULL,
    product_id BIGINT REFERENCES products(id) NOT NULL
);

INSERT INTO products_tax_identifiers (tax_processor_id, product_id)
SELECT
  'modrinth-servers' tax_processor_id,
  id product_id
FROM products
WHERE metadata ->> 'type' = 'pyro';

INSERT INTO products_tax_identifiers (tax_processor_id, product_id)
SELECT
  'modrinth-plus' tax_processor_id,
  id product_id
FROM products
WHERE metadata ->> 'type' = 'midas';

