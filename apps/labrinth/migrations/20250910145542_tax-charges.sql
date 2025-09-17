ALTER TABLE charges ADD COLUMN tax_amount BIGINT NOT NULL DEFAULT 0;
ALTER TABLE charges ADD COLUMN tax_platform_id TEXT;

ALTER TABLE products ADD COLUMN name TEXT;

CREATE TABLE products_tax_identifiers (
	id SERIAL PRIMARY KEY,
	tax_processor_id TEXT NOT NULL,
	product_id BIGINT REFERENCES products (id) NOT NULL
);

INSERT INTO products_tax_identifiers (tax_processor_id, product_id)
SELECT
	'modrinth-servers' AS tax_processor_id,
	id AS product_id
FROM products
WHERE metadata ->> 'type' = 'pyro';

INSERT INTO products_tax_identifiers (tax_processor_id, product_id)
SELECT
	'modrinth-plus' AS tax_processor_id,
	id AS product_id
FROM products
WHERE metadata ->> 'type' = 'midas';
