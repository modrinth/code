CREATE TABLE affiliate_codes (
	id            bigint      PRIMARY KEY,
	created_at    timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
	created_by    bigint      NOT NULL REFERENCES users(id),
	affiliate     bigint      NOT NULL REFERENCES users(id),
	-- left nullable so we can explicitly set payouts if we need to,
	-- and use a global default if unset
	revenue_split float
);
