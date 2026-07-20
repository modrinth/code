CREATE TABLE payouts_runs(
	period_start TIMESTAMPTZ PRIMARY KEY,
	started_at TIMESTAMPTZ NOT NULL,
	started_by BIGINT REFERENCES users(id)
		ON DELETE SET NULL,
	completed_at TIMESTAMPTZ,
	adjustments JSONB NOT NULL
);
