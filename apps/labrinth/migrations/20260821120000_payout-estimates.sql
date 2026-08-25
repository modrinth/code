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
