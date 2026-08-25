CREATE TABLE payout_estimates (
	period DATE NOT NULL
		CHECK (EXTRACT(DAY FROM period) = 1),
	user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	mod_id BIGINT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
	amount NUMERIC(40, 20) NOT NULL
		CHECK (amount >= 0),
	created TIMESTAMPTZ NOT NULL,
	date_available TIMESTAMPTZ NOT NULL,
	calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	PRIMARY KEY (period, user_id, mod_id)
);

CREATE INDEX payout_estimates_user_id
	ON payout_estimates (user_id);

CREATE INDEX payout_estimates_mod_id
	ON payout_estimates (mod_id);

CREATE INDEX payout_estimates_created
	ON payout_estimates (created);

ALTER TABLE payouts_values
	ADD COLUMN payout_run_id BIGINT REFERENCES payout_runs(id);

ALTER TABLE payouts_values
	ADD CONSTRAINT payouts_values_payout_run_creator_only
	CHECK (payout_run_id IS NULL OR mod_id IS NOT NULL);

CREATE UNIQUE INDEX payouts_values_payout_run_distribution
	ON payouts_values (payout_run_id, user_id, mod_id)
	WHERE payout_run_id IS NOT NULL;

DROP INDEX payout_runs_active_period;

CREATE UNIQUE INDEX payout_runs_single_active
	ON payout_runs ((TRUE))
	WHERE status IN ('scheduled', 'running');
