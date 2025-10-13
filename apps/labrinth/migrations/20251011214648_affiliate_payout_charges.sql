ALTER TABLE charges
ADD COLUMN affiliate_code BIGINT;

ALTER TABLE payouts_values
ADD COLUMN affiliate_code_id BIGINT;
