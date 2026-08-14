CREATE TABLE payouts_variance (
	applied_at TIMESTAMPTZ PRIMARY KEY,
	variance NUMERIC(40, 20) NOT NULL
		CHECK (variance BETWEEN 0 AND 1)
);

INSERT INTO payouts_variance (applied_at, variance)
VALUES ('1970-01-01 00:00:00+00', 0.1);
