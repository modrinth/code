CREATE TABLE payout_periods (
	period DATE PRIMARY KEY
		CHECK (EXTRACT(DAY FROM period) = 1),
	raw_actual_aditude_revenue_usd NUMERIC(40, 20) NOT NULL
		CHECK (raw_actual_aditude_revenue_usd >= 0),
	revenue_adjustments JSONB NOT NULL
);

CREATE TABLE payout_period_days (
	period DATE NOT NULL REFERENCES payout_periods(period),
	date DATE NOT NULL,
	raw_estimated_aditude_revenue_usd NUMERIC(40, 20) NOT NULL
		CHECK (raw_estimated_aditude_revenue_usd >= 0),
	aditude_impressions BIGINT NOT NULL
		CHECK (aditude_impressions >= 0),
	CHECK (date >= period AND date < period + INTERVAL '1 month'),
	PRIMARY KEY (period, date)
);

CREATE TABLE payout_runs (
	id BIGINT PRIMARY KEY,
	period DATE NOT NULL REFERENCES payout_periods(period),
	payload JSONB NOT NULL,
	status TEXT NOT NULL,
	started_at TIMESTAMPTZ NOT NULL,
	started_by BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	execute_at TIMESTAMPTZ NOT NULL,
	processing_started_at TIMESTAMPTZ,
	finished_at TIMESTAMPTZ,
	cancelled_at TIMESTAMPTZ,
	cancelled_by BIGINT REFERENCES users(id) ON DELETE CASCADE,
	error JSONB,
	CHECK (execute_at >= started_at),
	CHECK (
		processing_started_at IS NULL
		OR processing_started_at >= started_at
	),
	CHECK (finished_at IS NULL OR finished_at >= started_at),
	CHECK (cancelled_at IS NULL OR cancelled_at >= started_at)
);

CREATE UNIQUE INDEX payout_runs_active_period
	ON payout_runs (period)
	WHERE status = 'scheduled';

CREATE UNIQUE INDEX payout_runs_succeeded_period
	ON payout_runs (period)
	WHERE status = 'succeeded';

CREATE INDEX payout_runs_scheduled_execute_at
	ON payout_runs (execute_at)
	WHERE status = 'scheduled';

CREATE TABLE payouts_variance (
	applied_on DATE PRIMARY KEY,
	variance NUMERIC(40, 20) NOT NULL
		CHECK (variance BETWEEN 0 AND 1)
);

INSERT INTO payouts_variance (applied_on, variance)
VALUES ('1970-01-01', 0.1);
