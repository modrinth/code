CREATE TABLE payouts_runs(
	id BIGINT PRIMARY KEY,
	-- timestamp on the 1st of a month at midnight,
	-- representing what month this run is for.
	-- if a row exists for a month, then a payout run
	-- is running/has completed for this month (see
	-- `completed_at`).
	period_start TIMESTAMPTZ NOT NULL,
	started_at TIMESTAMPTZ NOT NULL,
	started_by BIGINT REFERENCES users(id)
		ON DELETE SET NULL,
	completed_at TIMESTAMPTZ,
	completed_result JSONB,
	adjustments JSONB NOT NULL
);
CREATE INDEX payouts_runs_period_start ON payouts_runs(period_start);
