CREATE TABLE users_subscriptions_affiliations (
	id				BIGSERIAL   NOT NULL PRIMARY KEY,
	subscription_id BIGINT      NOT NULL REFERENCES users_subscriptions(id),
	affiliate_code  BIGINT      NOT NULL REFERENCES affiliate_codes(id),
	deactivated_at  TIMESTAMPTZ
);

CREATE TABLE users_subscriptions_affiliations_payouts(
	id              BIGSERIAL          PRIMARY KEY,
	charge_id       BIGINT    NOT NULL REFERENCES charges(id),
	subscription_id BIGINT    NOT NULL REFERENCES users_subscriptions(id),
	affiliate_code  BIGINT    NOT NULL REFERENCES affiliate_codes(id),
	payout_value_id BIGSERIAL NOT NULL REFERENCES payouts_values(id),
	UNIQUE (charge_id)
);
