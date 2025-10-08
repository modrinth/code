CREATE TABLE affiliate_codes (
	id BIGINT PRIMARY KEY,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	created_by BIGINT NOT NULL REFERENCES users (id),
	affiliate BIGINT NOT NULL REFERENCES users (id),
	-- left nullable so we can explicitly set payouts if we need to,
	-- and use a global default if unset
	revenue_split FLOAT
);
