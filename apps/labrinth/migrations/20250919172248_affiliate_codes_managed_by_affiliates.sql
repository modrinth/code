ALTER TABLE affiliate_codes
ADD COLUMN source_name VARCHAR(255) NOT NULL DEFAULT '(unnamed)';
